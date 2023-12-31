use bevy::prelude::*;

pub mod animated_interaction;
pub mod flux_interaction;
pub mod highlight_background;
pub use flux_interaction::*;

use self::{
    animated_interaction::AnimatedInteractionPlugin,
    highlight_background::HighlightBackgroundPlugin,
};

pub struct SickleUiPlugin;

impl Plugin for SickleUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            FluxInteractionPlugin,
            AnimatedInteractionPlugin,
            HighlightBackgroundPlugin,
        ));
    }
}

#[derive(Component)]
pub struct PointerTracker {
    pub pointer_over: bool,
    pub enter_timer: Timer,
    pub exit_timer: Timer,
    pub pointer_delta: Vec2,
}
