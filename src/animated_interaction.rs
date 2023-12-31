use bevy::prelude::*;
use sickle_math::ease::{Ease, ValueEasing};

use crate::{FluxInteraction, FluxInteractionStopwatch};

use super::FluxInteractionUpdate;

pub struct AnimatedInteractionPlugin;

impl Plugin for AnimatedInteractionPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            AnimatedInteractionUpdate.after(FluxInteractionUpdate),
        )
        .add_systems(PreUpdate, add_animated_interaction_state)
        .add_systems(
            Update,
            update_animated_interaction_state.in_set(AnimatedInteractionUpdate),
        );
    }
}

#[derive(SystemSet, Clone, Hash, Debug, Eq, PartialEq)]
pub struct AnimatedInteractionUpdate;

pub enum AnimationProgress {
    Start,
    Inbetween(f32),
    End,
}

#[derive(Clone)]
pub struct AnimationConfig {
    pub duration: f32,
    pub easing: Ease,
    pub out_duration: Option<f32>,
    pub out_easing: Option<Ease>,
}

impl Default for AnimationConfig {
    fn default() -> Self {
        Self {
            duration: Default::default(),
            easing: Default::default(),
            out_duration: Default::default(),
            out_easing: Default::default(),
        }
    }
}

#[derive(Component)]
pub struct AnimatedInteractionState {
    pub progress: AnimationProgress,
}

impl Default for AnimatedInteractionState {
    fn default() -> Self {
        Self {
            progress: AnimationProgress::Start,
        }
    }
}

#[derive(Component)]
pub struct AnimatedInteraction {
    pub tween: AnimationConfig,
    pub hover: Option<AnimationConfig>,
    pub press: Option<AnimationConfig>,
    pub cancel: Option<AnimationConfig>,
    pub reset_delay: Option<f32>,
}

impl Default for AnimatedInteraction {
    fn default() -> Self {
        Self {
            tween: AnimationConfig {
                duration: 0.1,
                ..default()
            },
            hover: Default::default(),
            press: AnimationConfig {
                duration: 0.05,
                out_duration: Some(0.),
                ..default()
            }
            .into(),
            cancel: AnimationConfig {
                duration: 0.1,
                ..default()
            }
            .into(),
            reset_delay: Default::default(),
        }
    }
}

fn add_animated_interaction_state(
    mut commands: Commands,
    q_animated: Query<
        Entity,
        (
            With<FluxInteraction>,
            With<AnimatedInteraction>,
            Without<AnimatedInteractionState>,
        ),
    >,
) {
    for entity in &q_animated {
        commands
            .entity(entity)
            .insert(AnimatedInteractionState::default());
    }
}

fn update_animated_interaction_state(
    mut q_interaction: Query<(
        &AnimatedInteraction,
        &FluxInteraction,
        &FluxInteractionStopwatch,
        &mut AnimatedInteractionState,
    )>,
) {
    for (animation, interaction, stopwatch, mut animation_state) in &mut q_interaction {
        let (default, hover, press, cancel) = (
            animation.tween.clone(),
            animation.hover.clone(),
            animation.press.clone(),
            animation.cancel.clone(),
        );

        let elapsed = stopwatch.0.elapsed_secs();

        let progress = match *interaction {
            FluxInteraction::Pressed => {
                let tween = press.unwrap_or(default);
                let tween_time = tween.duration.max(0.);
                if tween_time == 0. {
                    AnimationProgress::End
                } else {
                    let tween_ratio = (elapsed / tween_time).clamp(0., 1.).ease(tween.easing);
                    AnimationProgress::Inbetween(tween_ratio)
                }
            }
            FluxInteraction::Released => {
                let tween = press.unwrap_or(default);
                let tween_time = tween.out_duration.unwrap_or(tween.duration).max(0.);
                if tween_time == 0. {
                    AnimationProgress::End
                } else {
                    let easing = tween.out_easing.unwrap_or(tween.easing);
                    let tween_ratio = (elapsed / tween_time).clamp(0., 1.).ease(easing);
                    AnimationProgress::Inbetween(tween_ratio)
                }
            }
            FluxInteraction::PressCanceled => {
                let tween = cancel.unwrap_or(default);
                let tween_time = tween.duration.max(0.);
                let reset_delay = animation.reset_delay.unwrap_or(tween_time);
                let reset_length = tween.out_duration.unwrap_or(tween_time);

                if elapsed < reset_delay {
                    AnimationProgress::Start
                } else {
                    if tween_time == 0. {
                        AnimationProgress::End
                    } else {
                        let easing = tween.out_easing.unwrap_or(tween.easing);
                        let tween_ratio = ((elapsed - reset_delay) / reset_length)
                            .clamp(0., 1.)
                            .ease(easing);
                        AnimationProgress::Inbetween(tween_ratio)
                    }
                }
            }
            FluxInteraction::PointerEnter => {
                let tween = hover.unwrap_or(default);
                let tween_time = tween.duration.max(0.);
                if tween_time == 0. {
                    AnimationProgress::End
                } else {
                    let tween_ratio = (elapsed / tween_time).clamp(0., 1.).ease(tween.easing);
                    AnimationProgress::Inbetween(tween_ratio)
                }
            }
            FluxInteraction::PointerLeave => {
                let tween = hover.unwrap_or(default);
                let tween_time = tween.out_duration.unwrap_or(tween.duration).max(0.);
                if tween_time == 0. {
                    AnimationProgress::End
                } else {
                    let easing = tween.out_easing.unwrap_or(tween.easing);
                    let tween_ratio = (elapsed / tween_time).clamp(0., 1.).ease(easing);
                    AnimationProgress::Inbetween(tween_ratio)
                }
            }
            _ => AnimationProgress::End,
        };

        animation_state.progress = progress;
    }
}
