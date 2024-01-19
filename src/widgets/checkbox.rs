use bevy::{ecs::system::EntityCommands, prelude::*, ui::FocusPolicy};
use sickle_math::ease::Ease;

use crate::{
    animated_interaction::{AnimatedInteraction, AnimationConfig},
    interactions::InteractiveBackground,
    ui_builder::UiBuilder,
    FluxInteraction, TrackedInteraction,
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
    pub checked: bool,
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
    fn spawn(
        parent: &'a mut ChildBuilder<'w, 's, '_>,
        label: Option<String>,
    ) -> Entity {
        let mut input = parent.spawn(Checkbox::checkbox_container_bundle());
        Checkbox::add_content(&mut input, label);

        input.id()
    }

    fn parentless(commands: &'a mut Commands<'w, 's>, label: Option<String>) -> Entity {
        let mut input = commands.spawn(Checkbox::checkbox_container_bundle());
        Checkbox::add_content(&mut input, label);

        input.id()
    }

    fn checkbox_container_bundle() -> impl Bundle {
        let tween = AnimationConfig {
            duration: 0.1,
            easing: Ease::OutExpo,
            ..default()
        };

        (
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
        )
    }

    fn add_content(input: &mut EntityCommands<'w, 's, 'a>, label: Option<String>) {
        let mut check_node: Entity = Entity::PLACEHOLDER;
        input.with_children(|parent| {
            parent
                .spawn(NodeBundle {
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
                        .spawn(NodeBundle {
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
                Checkbox::add_label(parent, label);
            }
        });

        input.insert(Checkbox {
            check_node,
            checked: false,
        });
    }

    fn add_label(parent: &'a mut ChildBuilder<'w, 's, '_>, label: String) -> Entity {
        parent
            .spawn(TextBundle {
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
            })
            .id()
    }
}

pub trait UiCheckboxExt<'w, 's> {
    fn checkbox<'a>(&'a mut self, label: Option<String>) -> EntityCommands<'w, 's, 'a>;
}

impl<'w, 's> UiCheckboxExt<'w, 's> for UiBuilder<'w, 's, '_> {
    fn checkbox<'a>(&'a mut self, label: Option<String>) -> EntityCommands<'w, 's, 'a> {
        let mut checkbox = Entity::PLACEHOLDER;

        if let Some(entity) = self.entity() {
            self.commands().entity(entity).with_children(|parent| {
                checkbox = Checkbox::spawn(parent, label);
            });
        } else {
            checkbox = Checkbox::parentless(self.commands(), label);
        }

        self.commands().entity(checkbox)
    }
}
