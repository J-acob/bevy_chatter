use bevy::{app::PluginGroupBuilder, prelude::*, render::camera::CameraPlugin};

mod ai;
mod camera;
mod ui;

pub struct ChatterPlugins;

impl PluginGroup for ChatterPlugins {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        let mut group = PluginGroupBuilder::start::<Self>();

        // Add plugins here
        group = group
            .add(self::ai::AiPlugin::default())
            .add(self::camera::CameraPlugin::default())
            .add(self::ui::UiPlugin::default());

        // Conditionally add plugins if need be.

        group
    }
}

pub mod prelude {
    pub use super::ai::*;
    pub use super::camera::*;
    pub use super::ui::*;
    pub use bevy::prelude::*;
}
