use std::{
    fs::File,
    io::{BufReader, ErrorKind, Read},
    sync::Arc,
};

use bevy::{
    audio::CpalSample,
    prelude::*,
    render::{renderer::{RenderAdapter, RenderDevice, RenderInstance, RenderQueue}, settings::Backends, render_resource::WgpuAdapterInfo},
    tasks::{AsyncComputeTaskPool, Task, TaskPool},
    utils::HashMap,
};

use itertools::Itertools;
use web_rwkv::{
    Environment,
    Model,
    //model::{Model, ModelState},
    //tokenizer::Tokenizer,
    Tokenizer, ModelState, Instance, Quantization, ModelBuilder
};

#[derive(Default)]
pub struct AiPlugin;

/// The current prompt being submitted to the AI
#[derive(Resource, Default, Deref, DerefMut)]
pub struct CurrentPrompt(pub String);

/// The current response of the AI
#[derive(Resource, Default, Deref, DerefMut)]
pub struct CurrentResponse(pub Option<String>);

/// Tokenizer(s)
#[derive(Resource, Default, Deref, DerefMut)]
pub struct AiTokenizer(pub Option<Tokenizer>);

#[derive(Resource, Default, Deref, DerefMut)]
pub struct AiEnvironment(pub Option<Environment>);

pub struct AiModel(pub Option<Model>);

#[derive(Resource, Default, Deref, DerefMut)]
pub struct AiModelState(pub Option<ModelState>);

#[derive(Resource, Default, Deref, DerefMut)]
pub struct AiError(pub Option<String>);

#[derive(States, Default, Debug, Clone, Hash, PartialEq, Eq)]
pub enum AiState {
    #[default]
    Waiting,
    Processing,
    Speaking,
}

#[derive(Resource, Debug, Clone)]
pub struct Sampler {
    top_p: f32,
    temp: f32,
    presence_penalty: f32,
    frequency_penalty: f32,
}

impl Default for Sampler {
    fn default() -> Self {
        Self {
            top_p: 0.5,
            temp: 1.0,
            presence_penalty: 0.3,
            frequency_penalty: 0.3,
        }
    }
}

impl Sampler {
    pub fn sample(&self, probs: Vec<f32>) -> u16 {
        let sorted: Vec<_> = probs
            .into_iter()
            .enumerate()
            .sorted_unstable_by(|(_, x), (_, y)| x.total_cmp(&y).reverse())
            .scan((0, 0.0, 0.0), |(_, cum, _), (id, x)| {
                if *cum > self.top_p {
                    None
                } else {
                    *cum += x;
                    Some((id, *cum, x))
                }
            })
            .map(|(id, _, x)| (id, x.powf(1.0 / self.temp)))
            .collect();

        let sum: f32 = sorted.iter().map(|(_, x)| x).sum();
        let sorted: Vec<_> = sorted
            .into_iter()
            .map(|(id, x)| (id, x / sum))
            .scan((0, 0.0), |(_, cum), (id, x)| {
                *cum += x;
                Some((id, *cum))
            })
            .collect();

        let rand = fastrand::f32();
        let token = sorted
            .into_iter()
            .find_or_first(|&(_, cum)| rand <= cum)
            .map(|(id, _)| id)
            .unwrap_or_default();
        token as u16
    }
}

#[derive(Event)]
pub struct AiPromptEvent {
    pub prompt: String,
}

// Events need to be updated in every frame in order to clear our buffers.
// This update should happen before we use the events.
// Here, we use system sets to control the ordering.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum AiPromptEvents {
    WritePrompt,
    ReadPrompt,
}

impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<AiState>()
            .add_event::<AiPromptEvent>()
            .insert_resource(CurrentResponse(None))
            .insert_resource(CurrentPrompt::default())
            .insert_resource(AiTokenizer(None))
            .insert_resource(AiEnvironment(None))
            .insert_resource(AiModelState(None))
            .insert_resource(AiError(None))
            .insert_non_send_resource(AiModel(None))
            .insert_resource(Sampler::default())
            .configure_sets(
                Update,
                (AiPromptEvents::WritePrompt, AiPromptEvents::ReadPrompt).chain(),
            )
            //.add_systems(Startup, (load_tokenizer))
            .add_systems(
                Startup,
                (load_tokenizer, create_environment, create_model, create_model_state).chain(),
            )
            .add_systems(Update, run_ai.in_set(AiPromptEvents::ReadPrompt))
            ;
    }
}

fn create_model_state(world: &mut World) {
    let _ = world.resource_scope(|world, mut model_state: Mut<AiModelState>| {
        // Get the model
        let Some(send_model) = world.get_non_send_resource::<AiModel>() else {
            return;
        };
        let Some(ref model) = send_model.0 else {
            return;
        };

        model_state.0 = Some(model.create_state());
    });
}


fn run_ai<'a>(
    mut ai_prompt_events: EventReader<AiPromptEvent>,
    ai_model: NonSend<AiModel>,
    ai_tokenizer: Res<AiTokenizer>,
    ai_model_state: Res<AiModelState>,
    sampler: Res<Sampler>,
    mut current_response: ResMut<CurrentResponse>,
) {
    // Gather the neccesary resources for running the ai
    let Some(model) = &ai_model.0 else { return };
    let Some(tokenizer) = &ai_tokenizer.0 else {
        return;
    };
    let Some(state) = &ai_model_state.0 else {
        return;
    };

    let user = "User";
    let bot = "Assistant";

    let max_tokens = 1024;

    // Iterate over prompts being sent
    for event in ai_prompt_events.iter() {
        let mut sentence = String::new();
        let mut occurences: bevy::utils::hashbrown::HashMap<u16, i32> = HashMap::new();

        // Encode the prompt
        let prompt = &event.prompt;

        let updated_prompt = format!("{user}: {prompt}\n\n{bot}:");

        let Ok(mut tokens) = tokenizer.encode(updated_prompt.as_bytes()) else {
            return;
        };

        // This is where the ai actually "runs"
        // NOTE: It's currently not possible to actually multithread this :/
        loop {
            let logits_result = model.run(&tokens, &state);
            let Ok(mut logits) = logits_result else {
                return;
            };

            logits[0] = f32::NEG_INFINITY;

            for (&token, &count) in occurences.iter() {
                let penalty = sampler.presence_penalty + count as f32 * sampler.frequency_penalty;
                logits[token as usize] -= penalty;
            }

            let Ok(probs) = model.softmax(&logits) else {
                continue;
            };

            let token = sampler.sample(probs);

            let Ok(decoded_tokens) = tokenizer.decode(&[token]) else {
                continue;
            };
            let word_result = String::from_utf8(decoded_tokens);

            if let Ok(word) = word_result {
                sentence += &word;
                //println!("Word: {:?}", word);
            }

            tokens = vec![token];
            let count = occurences.get(&token).unwrap_or(&1);
            occurences.insert(token, *count);

            if token == 0 || sentence.contains("\n\n") || tokens.len() >= max_tokens {
                break;
            }
        }

        current_response.0 = Some(sentence);
    }
}

fn create_environment(
    render_adapter: Res<RenderAdapter>,
    //adapter_info: WgpuAdapterInfo,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    //render_instance: Res<RenderInstance>,
    mut ai_environment: ResMut<AiEnvironment>,
) {

    //let instance = Instance::new();
    
    /* 
    let adapter = async {
        instance.adapter(bevy::render::settings::PowerPreference::HighPerformance).await;
    };
    */

    let environment = Environment {
        adapter: render_adapter.0.clone(),
        device: render_device.clone_device(),
        queue: render_queue.0.clone(),
    };

    ai_environment.0 = Some(environment)
}

fn create_model(world: &mut World) {
    let built_model: Result<Model, std::io::Error> =
        world.resource_scope(|world, mut ai_error: Mut<AiError>| {
            let Some(ai_environment) = world.get_resource::<AiEnvironment>() else {
                return Err(std::io::Error::new(
                    ErrorKind::Other,
                    "Unable to locate environment!",
                ));
            };

            let file = File::open("assets/models/RWKV-4-World-0.4B-v1-20230529-ctx4096.st")?;

            let Ok(map) = (unsafe { memmap2::Mmap::map(&file) }) else {
                return Err(std::io::Error::new(
                    ErrorKind::Other,
                    "Failed to map file to memory!",
                ));
            };

            let quantization = Quantization::default();

            let Some(ref environment) = ai_environment.0 else {
                return Err(std::io::Error::new(
                    ErrorKind::Other,
                    "Failed to get environment",
                ));
            };

            let model = ModelBuilder::new(environment.clone(), &map)
                .with_quantization(quantization)
                .build();

            let Ok(built_model) = model else {
                return Err(std::io::Error::new(
                    ErrorKind::Other,
                    "Failed to build model",
                ));
            };
            Ok(built_model)
        });

    //let Some(mut ai_error) = world.get_resource_mut::<AiError>() else {return};
    let Some(mut ai_model) = world.get_non_send_resource_mut::<AiModel>() else {
        return;
    };

    match built_model {
        Ok(model) => {
            println!("Successfully built model!");
            //world.insert_non_send_resource(model)
            ai_model.0 = Some(model);
        }
        Err(e) => {
            //ai_error.0 = Some(e.to_string());
        }
    }
}

fn load_tokenizer(mut ai_tokenizer: ResMut<AiTokenizer>) {
    let Ok(file) = File::open("assets/rwkv_vocab_v20230424.json") else {
        return;
    };
    let mut reader = BufReader::new(file);
    let mut contents = String::new();
    let Ok(result) = reader.read_to_string(&mut contents) else {
        return;
    };
    let Ok(tokenizer) = Tokenizer::new(&contents) else {
        return;
    };
    ai_tokenizer.0 = Some(tokenizer);
}
