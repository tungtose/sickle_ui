use bevy::{
    asset::io::{file::FileAssetReader, AssetSource},
    prelude::*,
};

pub mod animated_interaction;
pub mod dev_panels;
pub mod drag_interaction;
pub mod drop_interaction;
pub mod flux_interaction;
pub mod hierarchy_delay;
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
use hierarchy_delay::HierarchyDelayPlugin;
use interactions::InteractionsPlugin;
use resize_interaction::ResizeHandlePlugin;
use scroll_interaction::ScrollInteractionPlugin;
use widgets::WidgetsPlugin;

use self::animated_interaction::AnimatedInteractionPlugin;

pub struct SickleUiPlugin;

impl Plugin for SickleUiPlugin {
    fn build(&self, app: &mut App) {
        app // reads assets from the "other" folder, rather than the default "assets" folder
            .register_asset_source(
                // This is the "name" of the new source, used in asset paths.
                // Ex: "custom://path/to/sprite.png"
                "sickle_ui",
                // This is a repeatable source builder. You can configure readers, writers,
                // processed readers, processed writers, asset watchers, etc.
                AssetSource::build()
                    .with_reader(move || {
                        Box::new(FileAssetReader::new(String::from("../sickle_ui/assets")))
                    })
                    .with_processed_reader(move || {
                        Box::new(FileAssetReader::new(String::from("../sickle_ui/assets")))
                    }),
            )
            .add_plugins((
                AnimatedInteractionPlugin,
                DragInteractionPlugin,
                DropInteractionPlugin,
                HierarchyDelayPlugin,
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
