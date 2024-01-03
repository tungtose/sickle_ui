use bevy::prelude::*;
use sickle_macros::simple_interaction_for;

use crate::{
    animated_interaction::{add_animated_interaction_state, update_animated_interaction_state},
    interaction_utils::{
        add_interactive_state, update_controlled_component, update_transition_base_state,
        ComponentController, InteractionConfig, InteractionState,
    },
};

use super::animated_interaction::AnimatedInteractionUpdate;

pub struct InteractiveBorderPlugin;

impl Plugin for InteractiveBorderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            InteractiveBorderSize::default(),
            InteractiveBorderColor::default(),
            InteractiveMargin::default(),
        ));
    }
}

#[simple_interaction_for((Style, UiRect, "border"))]
pub struct InteractiveBorderSize;

#[simple_interaction_for((BorderColor, Color))]
pub struct InteractiveBorderColor;

#[simple_interaction_for((Style, UiRect, "margin"))]
pub struct InteractiveMargin;