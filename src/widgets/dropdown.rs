use bevy::prelude::*;
// use sickle_math::ease::Ease;

// use crate::{
//     animated_interaction::{AnimatedInteraction, AnimationConfig},
//     interactions::InteractiveBackground,
//     FluxInteraction, TrackedInteraction,
// };

pub struct DropdownPlugin;

impl Plugin for DropdownPlugin {
    fn build(&self, _app: &mut App) {
        // app.add_systems(
        //     Update,
        //     (
        //         toggle_radio_button,
        //         update_radio_group_buttons,
        //         update_radio_button,
        //     )
        //         .chain(),
        // );
    }
}

#[derive(Component, Default, Debug, Reflect)]
#[reflect(Component)]
pub struct Dropdown {}
