use bevy::prelude::*;

pub mod animated_interaction;
pub mod flux_interaction;
pub mod interactions;

pub use flux_interaction::*;
use interactions::InteractionsPlugin;

use self::animated_interaction::AnimatedInteractionPlugin;

pub struct SickleUiPlugin;

impl Plugin for SickleUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            FluxInteractionPlugin,
            AnimatedInteractionPlugin,
            InteractionsPlugin,
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
