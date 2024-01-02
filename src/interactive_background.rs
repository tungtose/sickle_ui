use bevy::prelude::*;
use sickle_math::lerp::Lerp;

use crate::{
    animated_interaction::{add_animated_interaction_state, update_animated_interaction_state},
    FluxInteraction,
};

use super::animated_interaction::{
    AnimatedInteractionState, AnimatedInteractionUpdate, AnimationProgress,
};

pub struct InteractiveBackgroundPlugin;

impl Plugin for InteractiveBackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (
                add_animated_interaction_state::<InteractiveBackground>,
                add_interactive_background_state,
            ),
        )
        .add_systems(
            Update,
            update_animated_interaction_state::<InteractiveBackground>
                .in_set(AnimatedInteractionUpdate),
        )
        .add_systems(
            Update,
            (update_transition_base_color, update_background_color)
                .chain()
                .after(AnimatedInteractionUpdate),
        );
    }
}

#[derive(Component)]
struct InteractiveBackgroundState {
    original: Color,
    transition_base: Color,
}

fn add_interactive_background_state(
    mut commands: Commands,
    q_interaction: Query<
        (Entity, &BackgroundColor),
        (
            With<InteractiveBackground>,
            With<FluxInteraction>,
            Without<InteractiveBackgroundState>,
        ),
    >,
) {
    for (entity, bg_color) in &q_interaction {
        commands.entity(entity).insert(InteractiveBackgroundState {
            original: bg_color.0,
            transition_base: bg_color.0,
        });
    }
}

fn update_transition_base_color(
    mut q_interaction: Query<
        (
            &BackgroundColor,
            &mut InteractiveBackgroundState,
            &FluxInteraction,
        ),
        Changed<FluxInteraction>,
    >,
) {
    for (bg_color, mut state, interaction) in &mut q_interaction {
        if *interaction == FluxInteraction::Pressed {
            state.transition_base = bg_color.0;
        }
    }
}

#[derive(Component)]
pub struct InteractiveBackground {
    pub highlight: Option<Color>,
    pub pressed: Option<Color>,
    pub cancel: Option<Color>,
}

impl Default for InteractiveBackground {
    fn default() -> Self {
        Self {
            highlight: Default::default(),
            pressed: Default::default(),
            cancel: Default::default(),
        }
    }
}

fn update_background_color(
    mut q_interaction: Query<(
        &InteractiveBackground,
        &InteractiveBackgroundState,
        &FluxInteraction,
        Option<&AnimatedInteractionState<InteractiveBackground>>,
        &mut BackgroundColor,
    )>,
) {
    for (highlight, highlight_state, interaction, animation_state, mut bg_color) in
        &mut q_interaction
    {
        let original_color = highlight_state.original;

        let (start_color, end_color) = match *interaction {
            FluxInteraction::Pressed => {
                let Some(pressed_color) = highlight.pressed else {
                    continue;
                };

                (Some(highlight_state.transition_base), pressed_color)
            }
            FluxInteraction::Released => {
                let end_color = highlight.highlight.unwrap_or(original_color);

                (highlight.pressed, end_color)
            }
            FluxInteraction::PressCanceled => (highlight.cancel, original_color),
            FluxInteraction::PointerEnter => {
                let Some(highlight_color) = highlight.highlight else {
                    continue;
                };

                (Some(original_color), highlight_color)
            }
            FluxInteraction::PointerLeave => {
                let Some(highlight_color) = highlight.highlight else {
                    continue;
                };

                (Some(highlight_color), original_color)
            }
            _ => (None, original_color),
        };

        let new_color = if let (Some(state), Some(start_color)) = (animation_state, start_color) {
            match state.progress {
                AnimationProgress::Start => start_color,
                AnimationProgress::Inbetween(tween_ratio) => {
                    start_color.lerp(end_color, tween_ratio)
                }
                AnimationProgress::End => end_color,
            }
        } else {
            end_color
        };

        bg_color.0 = new_color;
    }
}
