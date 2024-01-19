use bevy::{ecs::system::EntityCommands, prelude::*, ui::FocusPolicy};
use sickle_math::ease::Ease;

use crate::{
    animated_interaction::{AnimatedInteraction, AnimationConfig},
    interactions::InteractiveBackground,
    scroll_interaction::{ScrollAxis, Scrollable},
    ui_builder::{UiBuilder, UiBuilderExt},
    FluxInteraction, FluxInteractionUpdate, TrackedInteraction,
};

use super::{
    // floating_panel::FloatingPanel,
    prelude::{FloatingPanelConfig, FloatingPanelLayout, UiFloatingPanelExt},
    scroll_view::ScrollThrough,
};

pub struct DropdownPlugin;

impl Plugin for DropdownPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                handle_option_press,
                update_dropdown_label,
                handle_click_or_touch.after(FluxInteractionUpdate),
                update_dropdown_panel_visibility,
            )
                .chain(),
        );
    }
}

fn update_dropdown_label(
    mut q_dropdowns: Query<(&mut Dropdown, &DropdownOptions), Changed<Dropdown>>,
    mut q_text: Query<&mut Text>,
) {
    for (mut dropdown, options) in &mut q_dropdowns {
        let Ok(mut label) = q_text.get_mut(dropdown.button_label) else {
            continue;
        };

        if let Some(value) = dropdown.value {
            if value >= options.0.len() {
                dropdown.value = None;
            }
        }

        let text = if let Some(value) = dropdown.value {
            options.0[value].clone()
        } else {
            String::from("---")
        };

        label.sections = vec![TextSection::new(text, TextStyle::default())];
    }
}

fn handle_click_or_touch(
    r_mouse: Res<Input<MouseButton>>,
    r_touches: Res<Touches>,
    mut q_dropdowns: Query<(Entity, &mut Dropdown, &FluxInteraction)>,
) {
    if r_mouse.any_just_released([MouseButton::Left, MouseButton::Middle, MouseButton::Right])
        || r_touches.any_just_released()
    {
        let mut open: Option<Entity> = None;
        for (entity, _, interaction) in &mut q_dropdowns {
            if *interaction == FluxInteraction::Released {
                open = entity.into();
                break;
            }
        }

        for (entity, mut dropdown, _) in &mut q_dropdowns {
            if let Some(open_dropdown) = open {
                if entity == open_dropdown {
                    dropdown.is_open = !dropdown.is_open;
                } else if dropdown.is_open {
                    dropdown.is_open = false;
                }
            } else if dropdown.is_open {
                dropdown.is_open = false;
            }
        }
    }
}

fn handle_option_press(
    q_options: Query<(&DropdownOption, &FluxInteraction), Changed<FluxInteraction>>,
    mut q_dropdown: Query<&mut Dropdown>,
) {
    for (option, interaction) in &q_options {
        if *interaction == FluxInteraction::Released {
            let Ok(mut dropdown) = q_dropdown.get_mut(option.dropdown) else {
                continue;
            };

            dropdown.value = option.option.into();
        }
    }
}

fn update_dropdown_panel_visibility(
    mut q_panels: Query<(&DropdownPanel, &mut Style)>,
    q_dropdown: Query<Ref<Dropdown>>,
) {
    for (panel, mut style) in &mut q_panels {
        let Ok(dropdown) = q_dropdown.get(panel.dropdown) else {
            continue;
        };

        if !dropdown.is_changed() {
            continue;
        }

        if dropdown.is_open {
            style.display = Display::Flex;
        } else if style.display != Display::None {
            style.display = Display::None;
        }
    }
}

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct DropdownOptions(Vec<String>);

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct DropdownOption {
    dropdown: Entity,
    option: usize,
}

impl Default for DropdownOption {
    fn default() -> Self {
        Self {
            dropdown: Entity::PLACEHOLDER,
            option: Default::default(),
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct DropdownPanel {
    dropdown: Entity,
}

impl Default for DropdownPanel {
    fn default() -> Self {
        Self {
            dropdown: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct Dropdown {
    value: Option<usize>,
    panel: Entity,
    button_label: Entity,
    is_open: bool,
}

impl Default for Dropdown {
    fn default() -> Self {
        Self {
            value: Default::default(),
            panel: Entity::PLACEHOLDER,
            button_label: Entity::PLACEHOLDER,
            is_open: false,
        }
    }
}

impl<'w, 's, 'a> Dropdown {
    fn base_tween() -> AnimationConfig {
        AnimationConfig {
            duration: 0.1,
            easing: Ease::OutExpo,
            ..default()
        }
    }

    fn base_bundle(options: Vec<String>) -> impl Bundle {
        (
            ButtonBundle {
                style: Style {
                    min_width: Val::Px(150.),
                    min_height: Val::Px(26.),
                    align_self: AlignSelf::Start,
                    align_content: AlignContent::Center,
                    justify_content: JustifyContent::Start,
                    margin: UiRect::all(Val::Px(5.)),
                    padding: UiRect::horizontal(Val::Px(5.)),
                    ..default()
                },
                background_color: Color::GRAY.into(),
                ..default()
            },
            TrackedInteraction::default(),
            InteractiveBackground {
                highlight: Some(Color::rgba(0., 1., 1., 0.3)),
                ..default()
            },
            AnimatedInteraction::<InteractiveBackground> {
                tween: Dropdown::base_tween(),
                ..default()
            },
            DropdownOptions(options),
        )
    }

    fn label_bundle() -> impl Bundle {
        TextBundle {
            style: Style {
                align_self: AlignSelf::Center,
                margin: UiRect::right(Val::Px(10.)),
                ..default()
            },
            focus_policy: FocusPolicy::Pass,
            ..default()
        }
    }

    fn option(
        parent: &'a mut ChildBuilder<'w, 's, '_>,
        option: usize,
        label: String,
        dropdown: Entity,
    ) {
        parent
            .spawn((
                ButtonBundle {
                    style: Style {
                        height: Val::Px(26.),
                        justify_content: JustifyContent::Start,
                        align_content: AlignContent::Center,
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
                AnimatedInteraction::<InteractiveBackground> {
                    tween: Dropdown::base_tween(),
                    ..default()
                },
                DropdownOption { dropdown, option },
                ScrollThrough,
                Scrollable::default(),
            ))
            .with_children(|parent| {
                parent.spawn(TextBundle {
                    style: Style {
                        align_self: AlignSelf::Center,
                        margin: UiRect::horizontal(Val::Px(10.)),
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
            });
    }
}

pub trait UiDropdownExt<'w, 's> {
    fn dropdown<'a>(&'a mut self, options: Vec<String>) -> EntityCommands<'w, 's, 'a>;
}

impl<'w, 's> UiDropdownExt<'w, 's> for UiBuilder<'w, 's, '_> {
    fn dropdown<'a>(&'a mut self, options: Vec<String>) -> EntityCommands<'w, 's, 'a> {
        let mut dropdown = Entity::PLACEHOLDER;
        let mut selected = Entity::PLACEHOLDER;

        if let Some(entity) = self.entity() {
            self.commands().entity(entity).with_children(|parent| {
                dropdown = parent
                    .spawn(Dropdown::base_bundle(options.clone()))
                    .with_children(|parent| {
                        selected = parent.spawn(Dropdown::label_bundle()).id();
                    })
                    .id();
            });
        } else {
            dropdown = self
                .commands()
                .spawn(Dropdown::base_bundle(options.clone()))
                .with_children(|parent| {
                    selected = parent.spawn(Dropdown::label_bundle()).id();
                })
                .id();
        }

        let mut new_entity = self.commands().entity(dropdown);
        let mut new_builder = new_entity.ui_builder();

        let panel_id = new_builder
            .floating_panel(
                FloatingPanelConfig {
                    draggable: false,
                    resizable: false,
                    restrict_scroll: ScrollAxis::Vertical.into(),
                    ..default()
                },
                FloatingPanelLayout {
                    size: Vec2 { x: 200., y: 100. },
                    position: None,
                    hidden: true,
                },
                |container| {
                    let Ok(mut entity_commands) = container.entity_commands() else {
                        return;
                    };

                    entity_commands.with_children(|parent| {
                        for (index, label) in options.iter().enumerate() {
                            Dropdown::option(parent, index, label.clone(), dropdown);
                        }
                    });
                },
            )
            .insert(DropdownPanel { dropdown })
            .id();

        let mut entity_commands = self.commands().entity(dropdown);

        entity_commands.insert(Dropdown {
            button_label: selected,
            panel: panel_id,
            ..default()
        });

        entity_commands
    }
}
