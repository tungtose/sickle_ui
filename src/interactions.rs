use bevy::prelude::*;
use sickle_macros::simple_interaction_for;
use sickle_math::lerp::Lerp;

use crate::{animated_interaction::*, FluxInteraction};

pub struct InteractionsPlugin;

impl Plugin for InteractionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            InteractiveBackground::default(),
            InteractiveBorderSize::default(),
            InteractiveBorderColor::default(),
            InteractiveMargin::default(),
            InteractiveHeight::default(),
        ));
    }
}

#[simple_interaction_for((BackgroundColor, Color))]
pub struct InteractiveBackground;

#[simple_interaction_for((Style, UiRect, "border"))]
pub struct InteractiveBorderSize;

#[simple_interaction_for((BorderColor, Color))]
pub struct InteractiveBorderColor;

#[simple_interaction_for((Style, UiRect, "margin"))]
pub struct InteractiveMargin;

#[simple_interaction_for((Style, Val, "height"))]
pub struct InteractiveHeight;

pub trait InteractionConfig {
    type TargetType;
    fn new(
        highlight: Option<Self::TargetType>,
        pressed: Option<Self::TargetType>,
        cancel: Option<Self::TargetType>,
    ) -> Self;
    fn highlight(&self) -> Option<Self::TargetType>;
    fn pressed(&self) -> Option<Self::TargetType>;
    fn cancel(&self) -> Option<Self::TargetType>;
}

pub trait InteractionState {
    type TargetType;

    fn original(&self) -> Self::TargetType;
    fn transition_base(&self) -> Self::TargetType;
    fn set_transition_base(&mut self, from: Self::TargetType);
}

pub trait ComponentController {
    type TargetType;
    type InteractionState;
    type ControlledComponent;

    fn state(from: &Self::ControlledComponent) -> Self::InteractionState;
    fn extract_value(from: &Self::ControlledComponent) -> Self::TargetType;
    fn update_controlled_component(
        controlled_component: Mut<'_, Self::ControlledComponent>,
        new_value: Self::TargetType,
    );
}

pub fn add_interactive_state<Interaction, State, ControlledComponent>(
    mut commands: Commands,
    q_interaction: Query<
        (Entity, &ControlledComponent),
        (With<Interaction>, With<FluxInteraction>, Without<State>),
    >,
) where
    State: Component + InteractionState,
    Interaction: Component
        + InteractionConfig
        + ComponentController<InteractionState = State, ControlledComponent = ControlledComponent>,
    ControlledComponent: Component,
{
    for (entity, source) in &q_interaction {
        commands.entity(entity).insert(Interaction::state(source));
    }
}

pub fn update_transition_base_state<Interaction, State, ControlledComponent, TargetType>(
    mut q_interaction: Query<
        (&ControlledComponent, &mut State, &FluxInteraction),
        Changed<FluxInteraction>,
    >,
) where
    State: Component + InteractionState<TargetType = TargetType>,
    Interaction: ComponentController<
        InteractionState = State,
        TargetType = TargetType,
        ControlledComponent = ControlledComponent,
    >,
    ControlledComponent: Component,
    TargetType: Lerp,
{
    for (controlled_component, mut state, interaction) in &mut q_interaction {
        if *interaction == FluxInteraction::Pressed {
            state.set_transition_base(Interaction::extract_value(controlled_component));
        }
    }
}

pub fn update_controlled_component<Interaction, State, ControlledComponent, TransitionType>(
    mut q_interaction: Query<(
        &Interaction,
        &State,
        &FluxInteraction,
        Option<&AnimatedInteractionState<Interaction>>,
        &mut ControlledComponent,
    )>,
) where
    Interaction: Component
        + InteractionConfig<TargetType = TransitionType>
        + ComponentController<
            InteractionState = State,
            TargetType = TransitionType,
            ControlledComponent = ControlledComponent,
        >,
    State: Component + InteractionState<TargetType = TransitionType>,
    ControlledComponent: Component,
    TransitionType: Lerp,
{
    for (highlight, highlight_state, interaction, state, controlled_component) in &mut q_interaction
    {
        if let Some(new_value) =
            calculate_interaction_result(highlight, highlight_state, interaction, state)
        {
            Interaction::update_controlled_component(controlled_component, new_value);
        }
    }
}

pub fn calculate_interaction_result<T, S, R>(
    config: &T,
    transient_state: &S,
    interaction: &FluxInteraction,
    animation_state: Option<&AnimatedInteractionState<T>>,
) -> Option<R>
where
    T: Component + InteractionConfig<TargetType = R>,
    S: InteractionState<TargetType = R>,
    R: Lerp,
{
    let original_value = transient_state.original();

    let (start_value, end_value) = match *interaction {
        FluxInteraction::Pressed => {
            let Some(pressed_value) = config.pressed() else {
                return None;
            };

            (Some(transient_state.transition_base()), pressed_value)
        }
        FluxInteraction::Released => {
            let end_value = config.highlight().unwrap_or(original_value);

            (config.pressed(), end_value)
        }
        FluxInteraction::PressCanceled => (config.cancel(), original_value),
        FluxInteraction::PointerEnter => {
            let Some(highlight_value) = config.highlight() else {
                return None;
            };

            (Some(original_value), highlight_value)
        }
        FluxInteraction::PointerLeave => {
            let Some(highlight_color) = config.highlight() else {
                return None;
            };

            (Some(highlight_color), original_value)
        }
        _ => (None, original_value),
    };

    let new_value = if let (Some(state), Some(start_value)) = (animation_state, start_value) {
        match state.progress {
            AnimationProgress::Start => start_value,
            AnimationProgress::Inbetween(tween_ratio) => start_value.lerp(end_value, tween_ratio),
            AnimationProgress::End => end_value,
        }
    } else {
        end_value
    };

    Some(new_value)
}
