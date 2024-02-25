use bevy::prelude::*;

pub mod animated_interaction;
pub mod drag_interaction;
pub mod drop_interaction;
pub mod flux_interaction;
pub mod input_extension;
pub mod interactions;
pub mod resize_interaction;
pub mod scroll_interaction;
pub mod ui_builder;
pub mod ui_commands;
pub mod ui_style;
pub mod widgets;

use drag_interaction::DragInteractionPlugin;
use drop_interaction::DropInteractionPlugin;
pub use flux_interaction::*;
use interactions::InteractionsPlugin;
use resize_interaction::ResizeHandlePlugin;
use scroll_interaction::ScrollInteractionPlugin;
use widgets::WidgetsPlugin;

use self::animated_interaction::AnimatedInteractionPlugin;

pub struct SickleUiPlugin;

impl Plugin for SickleUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            AnimatedInteractionPlugin,
            DragInteractionPlugin,
            DropInteractionPlugin,
            FluxInteractionPlugin,
            InteractionsPlugin,
            ResizeHandlePlugin,
            ScrollInteractionPlugin,
            WidgetsPlugin,
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
