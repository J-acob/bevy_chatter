use std::sync::Arc;

use bevy::{
    prelude::*,
    render::renderer::{RenderAdapter, RenderContext, RenderInstance},
};
use bevy_egui::{
    egui::{Align2, Area, Label},
    *,
};

use crate::prelude::{
    AiEnvironment, AiError, AiModel, AiPromptEvent, AiPromptEvents, CurrentPrompt, CurrentResponse,
};

#[derive(Default)]
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .add_systems(Update, chat.in_set(AiPromptEvents::WritePrompt));
    }
}

fn chat(
    mut contexts: EguiContexts,
    mut prompt: ResMut<CurrentPrompt>,
    response: Res<CurrentResponse>,
    mut prompt_events: EventWriter<AiPromptEvent>,
    model: NonSend<AiModel>,
    environment: Res<AiEnvironment>,
    ai_error: Res<AiError>,
) {
    let ctx = contexts.ctx_mut();

    egui::SidePanel::left("side_panel")
        .default_width(200.0)
        .show(ctx, |ui| {
            ui.heading("Options");

            match &model.0 {
                Some(n) => {
                    ui.label("Model loaded");
                }
                None => {
                    ui.label("Model not loaded");
                }
            }

            match environment.0 {
                Some(_) => {
                    ui.label("Environment loaded");
                }
                None => {
                    ui.label("Environment not loaded");
                }
            }

            ui.heading("Error(s):");
            match &ai_error.0 {
                Some(e) => {
                    ui.label(e);
                }
                None => {}
            }

            ui.allocate_space(egui::Vec2::new(1.0, 100.0));

            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.add(egui::Hyperlink::from_label_and_url(
                    "powered by egui",
                    "https://github.com/emilk/egui/",
                ));
            });
        });

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Chat");

        ui.horizontal(|ui| {
            let name_label = ui.label("Prompt: ");
            ui.text_edit_singleline(&mut prompt.0)
                .labelled_by(name_label.id);
            if ui.button("Submit").clicked() {
                // Do some sort of submission here!
                //info!("Hello!");
                prompt_events.send(AiPromptEvent {
                    prompt: prompt.0.clone(),
                });
            }
        });
        ui.heading("Ai response: ");
        // The response of the AI
        if let Some(ai_response) = &response.0 {
            ui.label(ai_response);
        }
    });
}
