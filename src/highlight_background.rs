use bevy::prelude::*;
use sickle_math::lerp::Lerp;

use crate::FluxInteraction;

use super::animated_interaction::{
    AnimatedInteractionState, AnimatedInteractionUpdate, AnimationProgress,
};

pub struct HighlightBackgroundPlugin;

impl Plugin for HighlightBackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, add_highlight_background_state)
            .add_systems(
                Update,
                (update_base_color, update_background_color)
                    .chain()
                    .after(AnimatedInteractionUpdate),
            );
    }
}

#[derive(Component)]
struct HighlightBackgroundState {
    original_color: Color,
    base_color: Color,
}

fn add_highlight_background_state(
    mut commands: Commands,
    q_highlighted: Query<
        (Entity, &BackgroundColor),
        (
            With<HighlightBackground>,
            With<FluxInteraction>,
            Without<HighlightBackgroundState>,
        ),
    >,
) {
    for (entity, bg_color) in &q_highlighted {
        commands.entity(entity).insert(HighlightBackgroundState {
            original_color: bg_color.0,
            base_color: bg_color.0,
        });
    }
}

fn update_base_color(
    mut q_interaction: Query<
        (
            &BackgroundColor,
            &mut HighlightBackgroundState,
            &FluxInteraction,
        ),
        Changed<FluxInteraction>,
    >,
) {
    for (bg_color, mut highlight, interaction) in &mut q_interaction {
        if *interaction == FluxInteraction::Pressed {
            highlight.base_color = bg_color.0;
        }
    }
}

#[derive(Component)]
pub struct HighlightBackground {
    pub highlight_color: Option<Color>,
    pub pressed_color: Option<Color>,
    pub cancel_color: Option<Color>,
}

impl HighlightBackground {
    pub fn new(
        highlight_color: Option<Color>,
        pressed_color: Option<Color>,
        cancel_color: Option<Color>,
    ) -> Self {
        Self {
            highlight_color,
            pressed_color,
            cancel_color,
        }
    }
}

impl Default for HighlightBackground {
    fn default() -> Self {
        Self {
            highlight_color: Default::default(),
            pressed_color: Default::default(),
            cancel_color: Default::default(),
        }
    }
}

fn update_background_color(
    mut q_interaction: Query<(
        &HighlightBackground,
        &HighlightBackgroundState,
        &FluxInteraction,
        Option<&AnimatedInteractionState>,
        &mut BackgroundColor,
    )>,
) {
    for (highlight, highlight_state, interaction, animation_state, mut bg_color) in
        &mut q_interaction
    {
        let original_color = highlight_state.original_color;

        let (start_color, end_color) = match *interaction {
            FluxInteraction::Pressed => {
                let Some(pressed_color) = highlight.pressed_color else {
                    continue;
                };

                (Some(highlight_state.base_color), pressed_color)
            }
            FluxInteraction::Released => {
                let end_color = highlight.highlight_color.unwrap_or(original_color);

                (highlight.pressed_color, end_color)
            }
            FluxInteraction::PressCanceled => (highlight.cancel_color, original_color),
            FluxInteraction::PointerEnter => {
                let Some(highlight_color) = highlight.highlight_color else {
                    continue;
                };

                (Some(original_color), highlight_color)
            }
            FluxInteraction::PointerLeave => {
                let Some(highlight_color) = highlight.highlight_color else {
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
