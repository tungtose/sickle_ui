use bevy::{ecs::system::EntityCommands, prelude::*, ui::FocusPolicy};
use sickle_math::ease::Ease;

use crate::{
    animated_interaction::{AnimatedInteraction, AnimationConfig},
    interactions::InteractiveBackground,
    scroll_interaction::{ScrollAxis, Scrollable},
    FluxInteraction, FluxInteractionUpdate, TrackedInteraction,
};

use super::{floating_panel::{FloatingPanel, FloatingPanelConfig}, scroll_view::ScrollThrough};

pub struct DropdownPlugin;

impl Plugin for DropdownPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, spawn_dropdown_panel);
        app.add_systems(
            Update,
            (
                handle_option_press,
                update_dropdown_label,
                handle_click_or_touch.after(FluxInteractionUpdate),
                update_dropdown_panel_visibility,
                update_dropdown_panel_layout,
            )
                .chain(),
        );
    }
}

fn spawn_dropdown_panel(
    mut q_dropdowns: Query<(Entity, &mut Dropdown, &DropdownOptions), Added<Dropdown>>,
    mut commands: Commands,
) {
    for (entity, mut dropdown, options) in &mut q_dropdowns {
        let (panel_id, mut container) = FloatingPanel::open(
            &mut commands,
            FloatingPanelConfig {
                draggable: false,
                resizable: false,
                restrict_scroll: ScrollAxis::Vertical.into(),
                ..default()
            },
            Vec2 { x: 200., y: 100. },
            None,
            true,
        );
        container.with_children(|parent| {
            for (index, label) in options.0.iter().enumerate() {
                Dropdown::option(parent, index, label.clone(), entity);
            }
        });

        commands
            .entity(panel_id)
            .insert(DropdownPanel { dropdown: entity });

        dropdown.panel = panel_id;
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

fn update_dropdown_panel_layout(
    mut q_panels: Query<(&DropdownPanel, &mut FloatingPanel, &Style), Changed<Style>>,
    q_node: Query<(&Node, &GlobalTransform)>,
) {
    for (panel, mut container, style) in &mut q_panels {
        if style.display == Display::None {
            continue;
        }

        let Ok((node, transform)) = q_node.get(panel.dropdown) else {
            continue;
        };

        let size = Vec2 { x: 200., y: 50. };
        if container.size != size {
            container.size = Vec2 { x: 200., y: 50. };
        }

        let position = node.logical_rect(transform).min
            + Vec2 {
                x: 0.,
                y: node.size().y,
            };
        if container.position != position {
            container.position = position;
        }

        // let Ok(container_node) = q_node.get(container.) else {
        //     continue;
        // };
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
    pub fn spawn(
        parent: &'a mut ChildBuilder<'w, 's, '_>,
        options: Vec<String>,
    ) -> EntityCommands<'w, 's, 'a> {
        let mut dropdown = parent.spawn((
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
        ));

        let mut selected = Entity::PLACEHOLDER;
        dropdown.with_children(|parent| {
            selected = parent
                .spawn(TextBundle {
                    style: Style {
                        align_self: AlignSelf::Center,
                        margin: UiRect::right(Val::Px(10.)),
                        ..default()
                    },
                    focus_policy: FocusPolicy::Pass,
                    ..default()
                })
                .id();
        });

        dropdown.insert(Dropdown {
            button_label: selected,
            ..default()
        });

        dropdown
    }

    fn base_tween() -> AnimationConfig {
        AnimationConfig {
            duration: 0.1,
            easing: Ease::OutExpo,
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
                Scrollable::default()
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
