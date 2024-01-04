use bevy::{prelude::*, ui::FocusPolicy};
use sickle_math::ease::Ease;

use crate::{
    animated_interaction::{AnimatedInteraction, AnimationConfig},
    interactions::InteractiveBackground,
    FluxInteraction, TrackedInteraction,
};

use super::InputWidget;

pub struct CheckboxPlugin;

impl Plugin for CheckboxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, toggle_checkbox);
    }
}

fn toggle_checkbox(
    mut q_checkboxes: Query<(&mut InputCheckbox, &FluxInteraction), Changed<FluxInteraction>>,
    mut style: Query<&mut Style>,
) {
    for (mut checkbox, interaction) in &mut q_checkboxes {
        if *interaction == FluxInteraction::Released {
            checkbox.checked = !checkbox.checked;

            if let Ok(mut target) = style.get_mut(checkbox.check_node) {
                target.display = if checkbox.checked {
                    Display::Flex
                } else {
                    Display::None
                };
            }
        }
    }
}

#[derive(Component, Debug)]
pub struct InputCheckbox {
    checked: bool,
    check_node: Entity,
}

impl Default for InputCheckbox {
    fn default() -> Self {
        Self {
            checked: false,
            check_node: Entity::PLACEHOLDER,
        }
    }
}

impl InputWidget for InputCheckbox {
    fn spawn(builder: &mut ChildBuilder, label: Option<String>) {
        let tween = AnimationConfig {
            duration: 0.1,
            easing: Ease::OutExpo,
            ..default()
        };

        let mut input = builder.spawn((
            ButtonBundle {
                style: Style {
                    height: Val::Px(26.),
                    align_self: AlignSelf::Start,
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
                highlight: Some(Color::rgba(0., 1., 1., 0.3)),
                ..default()
            },
            AnimatedInteraction::<InteractiveBackground> { tween, ..default() },
        ));

        let mut check_node: Entity = Entity::PLACEHOLDER;
        input.with_children(|builder| {
            builder
                .spawn(ButtonBundle {
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
                })
                .with_children(|builder| {
                    check_node = builder
                        .spawn(ButtonBundle {
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
                        })
                        .id();
                });

            if let Some(label) = label {
                builder.spawn(TextBundle {
                    style: Style {
                        align_self: AlignSelf::Center,
                        margin: UiRect::right(Val::Px(10.)),
                        ..default()
                    },
                    text: Text::from_section(
                        label,
                        TextStyle {
                            color: Color::BLACK,
                            ..default()
                        },
                    ),
                    focus_policy: FocusPolicy::Pass,
                    ..default()
                });
            }
        });

        input.insert(InputCheckbox {
            check_node,
            checked: false,
        });
    }
}
