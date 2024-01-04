use bevy::{ecs::system::EntityCommands, prelude::*, ui::FocusPolicy};
use sickle_math::ease::Ease;

use crate::{
    animated_interaction::{AnimatedInteraction, AnimationConfig},
    interactions::InteractiveBackground,
    FluxInteraction, TrackedInteraction,
};

pub struct RadioGroupPlugin;

impl Plugin for RadioGroupPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                toggle_radio_button,
                update_radio_group_buttons,
                update_radio_button,
            )
                .chain(),
        );
    }
}

fn toggle_radio_button(
    mut q_radio_buttons: Query<(&mut InputRadioButton, &FluxInteraction), Changed<FluxInteraction>>,
    keys: Res<Input<KeyCode>>,
    mut q_group: Query<&mut InputRadioGroup>,
) {
    for (mut radio_button, interaction) in &mut q_radio_buttons {
        if *interaction == FluxInteraction::Released {
            let mut changed = false;

            if radio_button.checked
                && radio_button.unselectable
                && keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
            {
                radio_button.checked = false;
                changed = true;
            } else if !radio_button.checked {
                radio_button.checked = true;
                changed = true;
            }

            if !changed {
                continue;
            }

            if let Some(group) = radio_button.group {
                let Ok(mut radio_group) = q_group.get_mut(group) else {
                    continue;
                };

                radio_group.selected = if radio_button.checked {
                    radio_button.index.into()
                } else {
                    None
                };
            }
        }
    }
}

fn update_radio_group_buttons(
    mut q_radio_buttons: Query<(&InputRadioGroup, &Children), Changed<InputRadioGroup>>,
    mut q_radio_button: Query<&mut InputRadioButton>,
) {
    for (radio_group, children) in &mut q_radio_buttons {
        for child in children {
            if let Ok(mut button) = q_radio_button.get_mut(*child) {
                // This is to avoid double triggering the change
                let checked = radio_group.selected == button.index.into();
                if button.checked != checked {
                    button.checked = checked;
                }
            }
        }
    }
}

fn update_radio_button(
    q_checkboxes: Query<&InputRadioButton, Changed<InputRadioButton>>,
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
pub struct InputRadioGroup {
    pub selected: Option<usize>,
}

impl Default for InputRadioGroup {
    fn default() -> Self {
        Self { selected: None }
    }
}

impl<'w, 's, 'a> InputRadioGroup {
    pub fn spawn(
        parent: &'a mut ChildBuilder<'w, 's, '_>,
        options: Vec<Option<String>>,
        unselectable: bool,
    ) -> EntityCommands<'w, 's, 'a> {
        let mut group = parent.spawn((NodeBundle::default(), InputRadioGroup::default()));
        let id = Some(group.id());

        group.with_children(|builder| {
            for (index, label) in options.iter().enumerate() {
                InputRadioButton::spawn(
                    builder,
                    index.try_into().unwrap(),
                    label.clone(),
                    id,
                    unselectable,
                );
            }
        });

        group
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct InputRadioButton {
    pub index: usize,
    pub checked: bool,
    unselectable: bool,
    check_node: Entity,
    group: Option<Entity>,
}

impl Default for InputRadioButton {
    fn default() -> Self {
        Self {
            index: 0,
            checked: false,
            unselectable: false,
            check_node: Entity::PLACEHOLDER,
            group: None,
        }
    }
}

impl<'w, 's, 'a> InputRadioButton {
    pub fn spawn(
        parent: &'a mut ChildBuilder<'w, 's, '_>,
        index: usize,
        label: Option<String>,
        group: Option<Entity>,
        unselectable: bool,
    ) -> EntityCommands<'w, 's, 'a> {
        let tween = AnimationConfig {
            duration: 0.1,
            easing: Ease::OutExpo,
            ..default()
        };

        let mut input = parent.spawn((
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

        input.insert(InputRadioButton {
            index,
            checked: false,
            unselectable,
            check_node,
            group,
        });

        input
    }
}
