use bevy::{ecs::system::EntityCommands, prelude::*, ui::FocusPolicy};
use sickle_math::ease::Ease;

use crate::{
    animated_interaction::{AnimatedInteraction, AnimationConfig},
    interactions::InteractiveBackground,
    ui_builder::UiBuilder,
    FluxInteraction, TrackedInteraction,
};

use super::{
    label::LabelConfig,
    prelude::{UiContainerExt, UiLabelExt},
};

pub struct CheckboxPlugin;

impl Plugin for CheckboxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (toggle_checkbox, update_checkbox).chain());
    }
}

fn toggle_checkbox(
    mut q_checkboxes: Query<(&mut Checkbox, &FluxInteraction), Changed<FluxInteraction>>,
) {
    for (mut checkbox, interaction) in &mut q_checkboxes {
        if *interaction == FluxInteraction::Released {
            checkbox.checked = !checkbox.checked;
        }
    }
}

fn update_checkbox(
    q_checkboxes: Query<&Checkbox, Changed<Checkbox>>,
    mut style: Query<&mut Style>,
) {
    for checkbox in &q_checkboxes {
        if let Ok(mut target) = style.get_mut(checkbox.check_node) {
            target.display = if checkbox.checked {
                Display::Flex
            } else {
                Display::None
            };
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct Checkbox {
    checked: bool,
    check_node: Entity,
}

impl Default for Checkbox {
    fn default() -> Self {
        Self {
            checked: false,
            check_node: Entity::PLACEHOLDER,
        }
    }
}

impl<'w, 's, 'a> Checkbox {
    pub fn checked(&self) -> bool {
        self.checked
    }

    fn base_tween() -> AnimationConfig {
        AnimationConfig {
            duration: 0.1,
            easing: Ease::OutExpo,
            ..default()
        }
    }

    fn checkbox_container() -> impl Bundle {
        (
            ButtonBundle {
                style: Style {
                    height: Val::Px(26.),
                    align_content: AlignContent::Center,
                    justify_content: JustifyContent::Start,
                    margin: UiRect::all(Val::Px(5.)),
                    ..default()
                },
                background_color: Color::NONE.into(),
                ..default()
            },
            TrackedInteraction::default(),
            InteractiveBackground {
                highlight: Color::rgba(0., 1., 1., 0.3).into(),
                ..default()
            },
            AnimatedInteraction::<InteractiveBackground> {
                tween: Checkbox::base_tween(),
                ..default()
            },
        )
    }

    fn checkmark_background() -> impl Bundle {
        NodeBundle {
            style: Style {
                width: Val::Px(16.),
                height: Val::Px(16.),
                margin: UiRect::all(Val::Px(5.)),
                border: UiRect::all(Val::Px(1.)),
                ..default()
            },
            border_color: Color::DARK_GRAY.into(),
            focus_policy: FocusPolicy::Pass,
            ..default()
        }
    }

    fn checkmark() -> impl Bundle {
        NodeBundle {
            style: Style {
                display: Display::None,
                width: Val::Px(10.),
                height: Val::Px(10.),
                margin: UiRect::all(Val::Px(2.)),
                ..default()
            },
            background_color: Color::DARK_GRAY.into(),
            focus_policy: FocusPolicy::Pass,
            ..default()
        }
    }
}

pub trait UiCheckboxExt<'w, 's> {
    fn checkbox<'a>(&'a mut self, label: Option<impl Into<String>>) -> EntityCommands<'w, 's, 'a>;
}

impl<'w, 's> UiCheckboxExt<'w, 's> for UiBuilder<'w, 's, '_> {
    fn checkbox<'a>(&'a mut self, label: Option<impl Into<String>>) -> EntityCommands<'w, 's, 'a> {
        let mut check_node: Entity = Entity::PLACEHOLDER;

        let mut input = self.container(Checkbox::checkbox_container(), |container| {
            container.container(Checkbox::checkmark_background(), |checkmark_bg| {
                check_node = checkmark_bg.container(Checkbox::checkmark(), |_| {}).id();
            });

            if let Some(label) = label {
                container.label(LabelConfig {
                    label: label.into(),
                    margin: UiRect::right(Val::Px(10.)),
                    ..default()
                });
            }
        });

        input.insert(Checkbox {
            check_node,
            checked: false,
        });

        input
    }
}
