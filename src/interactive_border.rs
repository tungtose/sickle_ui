use bevy::prelude::*;

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
        ));
    }
}

impl Plugin for InteractiveBorderSize {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (
                add_animated_interaction_state::<InteractiveBorderSize>,
                add_interactive_state::<InteractiveBorderSize, InteractiveBorderSizeState, Style>,
            ),
        )
        .add_systems(
            Update,
            update_animated_interaction_state::<InteractiveBorderSize>
                .in_set(AnimatedInteractionUpdate),
        )
        .add_systems(
            Update,
            (
                update_transition_base_state::<
                    InteractiveBorderSize,
                    InteractiveBorderSizeState,
                    Style,
                    UiRect,
                >,
                update_controlled_component::<
                    InteractiveBorderSize,
                    InteractiveBorderSizeState,
                    Style,
                    UiRect,
                >,
            )
                .chain()
                .after(AnimatedInteractionUpdate),
        );
    }
}

impl Plugin for InteractiveBorderColor {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (
                add_animated_interaction_state::<InteractiveBorderColor>,
                add_interactive_state::<
                    InteractiveBorderColor,
                    InteractiveBorderColorState,
                    BorderColor,
                >,
            ),
        )
        .add_systems(
            Update,
            update_animated_interaction_state::<InteractiveBorderColor>
                .in_set(AnimatedInteractionUpdate),
        )
        .add_systems(
            Update,
            (
                update_transition_base_state::<
                    InteractiveBorderColor,
                    InteractiveBorderColorState,
                    BorderColor,
                    Color,
                >,
                update_controlled_component::<
                    InteractiveBorderColor,
                    InteractiveBorderColorState,
                    BorderColor,
                    Color,
                >,
            )
                .chain()
                .after(AnimatedInteractionUpdate),
        );
    }
}

#[derive(Component, Default)]
pub struct InteractiveBorderSize {
    pub highlight: Option<UiRect>,
    pub pressed: Option<UiRect>,
    pub cancel: Option<UiRect>,
}

impl InteractionConfig for InteractiveBorderSize {
    type TargetType = UiRect;

    fn highlight(&self) -> Option<Self::TargetType> {
        self.highlight
    }

    fn pressed(&self) -> Option<Self::TargetType> {
        self.pressed
    }

    fn cancel(&self) -> Option<Self::TargetType> {
        self.cancel
    }
}

impl ComponentController for InteractiveBorderSize {
    type TargetType = UiRect;
    type InteractionState = InteractiveBorderSizeState;
    type ControlledComponent = Style;

    fn state(from: &Self::ControlledComponent) -> Self::InteractionState {
        Self::InteractionState {
            original: Self::extract_value(from),
            transition_base: Self::extract_value(from),
        }
    }

    fn extract_value(from: &Self::ControlledComponent) -> Self::TargetType {
        from.border
    }

    fn update_controlled_component(
        mut controlled_component: Mut<'_, Self::ControlledComponent>,
        new_value: Self::TargetType,
    ) {
        controlled_component.border = new_value;
    }
}

#[derive(Component)]
pub struct InteractiveBorderSizeState {
    original: UiRect,
    transition_base: UiRect,
}

impl InteractionState for InteractiveBorderSizeState {
    type TargetType = UiRect;

    fn original(&self) -> Self::TargetType {
        self.original
    }
    fn transition_base(&self) -> Self::TargetType {
        self.transition_base
    }
    fn set_transition_base(&mut self, from: Self::TargetType) {
        self.transition_base = from;
    }
}

#[derive(Component, Default)]
pub struct InteractiveBorderColor {
    pub highlight: Option<Color>,
    pub pressed: Option<Color>,
    pub cancel: Option<Color>,
}

impl InteractionConfig for InteractiveBorderColor {
    type TargetType = Color;

    fn highlight(&self) -> Option<Self::TargetType> {
        self.highlight
    }

    fn pressed(&self) -> Option<Self::TargetType> {
        self.pressed
    }

    fn cancel(&self) -> Option<Self::TargetType> {
        self.cancel
    }
}

impl ComponentController for InteractiveBorderColor {
    type TargetType = Color;
    type InteractionState = InteractiveBorderColorState;
    type ControlledComponent = BorderColor;

    fn state(from: &Self::ControlledComponent) -> Self::InteractionState {
        Self::InteractionState {
            original: Self::extract_value(from),
            transition_base: Self::extract_value(from),
        }
    }

    fn extract_value(from: &Self::ControlledComponent) -> Self::TargetType {
        from.0
    }

    fn update_controlled_component(
        mut controlled_component: Mut<'_, Self::ControlledComponent>,
        new_value: Self::TargetType,
    ) {
        controlled_component.0 = new_value;
    }
}

#[derive(Component)]
pub struct InteractiveBorderColorState {
    original: Color,
    transition_base: Color,
}

impl InteractionState for InteractiveBorderColorState {
    type TargetType = Color;

    fn original(&self) -> Self::TargetType {
        self.original
    }
    fn transition_base(&self) -> Self::TargetType {
        self.transition_base
    }
    fn set_transition_base(&mut self, from: Self::TargetType) {
        self.transition_base = from;
    }
}
