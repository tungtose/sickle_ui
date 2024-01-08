use bevy::{ecs::system::EntityCommands, prelude::*, ui::FocusPolicy};
use sickle_math::{ease::Ease, lerp::Lerp};

use crate::{
    animated_interaction::{AnimatedInteraction, AnimationConfig},
    drag_interaction::{DragState, Draggable},
    interactions::InteractiveBackground,
    TrackedInteraction,
};

pub struct InputSliderPlugin;

impl Plugin for InputSliderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_slider_on_drag,
                update_slider_handle,
                update_slider_readout,
                // debug_slider,
                // debug_slider_handle,
            )
                .chain(),
        );
    }
}

// fn debug_slider(q_slider: Query<&InputSlider, Changed<InputSlider>>) {
//     for slider in &q_slider {
//         println!("{:?}", slider);
//     }
// }

// fn debug_slider_handle(q_slider: Query<&InputSlider, Changed<Style>>) {
//     for slider in &q_slider {
//         println!("{:?}", slider);
//     }
// }

// TODO: Add mouse wheel scroll (ScrollInteraction Vertical/Horizontal)
// TODO: Remove hardcoded theming
// TODO: Add input for value (w/ read/write flags)

fn update_slider_on_drag(
    q_draggable: Query<(&Draggable, &InputSliderDragHandle, &Node), Changed<Draggable>>,
    q_transform: Query<&Node>,
    mut q_slider: Query<&mut InputSlider>,
) {
    for (draggable, handle, node) in &q_draggable {
        if draggable.state == DragState::Inactive
            || draggable.state == DragState::MaybeDragged
            || draggable.state == DragState::DragCanceled
        {
            continue;
        }

        let mut slider = q_slider.get_mut(handle.slider).unwrap();
        let Ok(slider_bar) = q_transform.get(slider.slider_bar) else {
            continue;
        };

        let Some(diff) = draggable.diff else {
            continue;
        };

        let width = slider_bar.size().x - node.size().x;
        if diff.x == 0. || width == 0. {
            continue;
        }

        let fraction = diff.x / width;
        let ratio = (slider.ratio + fraction).clamp(0., 1.);

        slider.ratio = ratio;
    }
}

fn update_slider_handle(
    q_slider: Query<&InputSlider, Or<(Changed<InputSlider>, Changed<Node>)>>,
    q_transform: Query<&Node>,
    mut q_hadle_style: Query<(&Node, &mut Style), With<InputSliderDragHandle>>,
) {
    for slider in &q_slider {
        let Ok(slider_bar) = q_transform.get(slider.slider_bar) else {
            continue;
        };
        let Ok((node, mut style)) = q_hadle_style.get_mut(slider.drag_handle) else {
            continue;
        };

        let width = slider_bar.size().x - node.size().x;
        let handle_position = width * slider.ratio;
        if style.left != Val::Px(handle_position) {
            style.left = Val::Px(handle_position);
        }
    }
}

fn update_slider_readout(
    q_slider: Query<&InputSlider, Changed<InputSlider>>,
    mut q_style: Query<&mut Style>,
    mut q_text: Query<&mut Text>,
) {
    for slider in &q_slider {
        let Ok(mut text) = q_text.get_mut(slider.current_value_node) else {
            continue;
        };

        let Ok(mut style) = q_style.get_mut(slider.current_value_node) else {
            continue;
        };

        if slider.config.show_current {
            if style.display == Display::None {
                style.display = Display::Flex;
            }

            let content = format!("{:.1}", slider.value());
            let section = TextSection {
                value: content,
                style: TextStyle {
                    color: Color::BLACK,
                    font_size: 14.,
                    ..default()
                },
            };

            text.sections = vec![section];
        } else if !slider.config.show_current && style.display == Display::Flex {
            style.display = Display::None;
        }
    }
}

#[derive(Component, Debug, Reflect)]
pub struct SliderConfig {
    min: f32,
    max: f32,
    initial_value: f32,
    show_current: bool,
}

impl SliderConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from(min: f32, max: f32, initial_value: f32, show_current: bool) -> Self {
        if max <= min || initial_value < min || initial_value > max {
            panic!(
                "Invalid slider config values! Min: {}, Max: {}, Initial: {}",
                min, max, initial_value
            );
        }

        SliderConfig {
            min,
            max,
            initial_value,
            show_current,
        }
    }

    pub fn with_value(self, value: f32) -> Self {
        if value >= self.min && value <= self.max {
            return Self {
                initial_value: value,
                ..self
            };
        }

        panic!("Value must be between min and max!");
    }
}

impl Default for SliderConfig {
    fn default() -> Self {
        Self {
            min: 0.,
            max: 1.,
            initial_value: 0.5,
            show_current: Default::default(),
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct InputSlider {
    pub ratio: f32,
    pub config: SliderConfig,
    pub slider_bar: Entity,
    pub drag_handle: Entity,
    pub current_value_node: Entity,
}

impl Default for InputSlider {
    fn default() -> Self {
        Self {
            ratio: Default::default(),
            config: Default::default(),
            slider_bar: Entity::PLACEHOLDER,
            drag_handle: Entity::PLACEHOLDER,
            current_value_node: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct InputSliderDragHandle {
    pub slider: Entity,
}

impl Default for InputSliderDragHandle {
    fn default() -> Self {
        Self {
            slider: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Default, Debug, Reflect)]
pub enum SliderDirection {
    #[default]
    Horizontal,
    Vertical,
}

impl<'w, 's, 'a> InputSlider {
    pub fn value(&self) -> f32 {
        self.config.min.lerp(self.config.max, self.ratio)
    }

    pub fn spawn(
        parent: &'a mut ChildBuilder<'w, 's, '_>,
        label: Option<String>,
        config: Option<SliderConfig>,
    ) -> EntityCommands<'w, 's, 'a> {
        let tween = AnimationConfig {
            duration: 0.1,
            easing: Ease::OutExpo,
            ..default()
        };

        let mut input = parent.spawn((
            NodeBundle {
                style: Style {
                    height: Val::Px(20.),
                    justify_content: JustifyContent::SpaceBetween,
                    justify_self: JustifySelf::Stretch,
                    align_content: AlignContent::Center,
                    margin: UiRect::all(Val::Px(5.)),
                    ..default()
                },
                ..default()
            },
            TrackedInteraction::default(),
            InteractiveBackground {
                highlight: Some(Color::rgba(0., 1., 1., 0.8)),
                ..default()
            },
            AnimatedInteraction::<InteractiveBackground> { tween, ..default() },
        ));

        let input_id = input.id();
        let mut drag_handle: Entity = Entity::PLACEHOLDER;
        let mut slider_bar: Entity = Entity::PLACEHOLDER;
        let mut current_value_node: Entity = Entity::PLACEHOLDER;
        input.with_children(|parent| {
            if let Some(label) = label {
                parent.spawn(TextBundle {
                    style: Style {
                        align_self: AlignSelf::Center,
                        margin: UiRect::px(5., 10., 0., 0.),
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

            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.),
                        justify_content: JustifyContent::SpaceBetween,
                        align_self: AlignSelf::Stretch,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    slider_bar = parent
                        .spawn(NodeBundle {
                            style: Style {
                                width: Val::Percent(100.),
                                height: Val::Px(4.),
                                justify_self: JustifySelf::Stretch,
                                margin: UiRect::vertical(Val::Px(8.)),
                                border: UiRect::px(1., 1., 0., 1.),
                                ..default()
                            },
                            background_color: Color::DARK_GRAY.into(),
                            border_color: Color::GRAY.into(),
                            ..default()
                        })
                        .with_children(|parent| {
                            drag_handle = parent
                                .spawn((
                                    ButtonBundle {
                                        style: Style {
                                            width: Val::Px(20.),
                                            height: Val::Px(20.),
                                            align_self: AlignSelf::Stretch,
                                            margin: UiRect::top(Val::Px(-8.)),
                                            border: UiRect::px(1., 1., 1., 2.),
                                            ..default()
                                        },
                                        background_color: Color::AQUAMARINE.into(),
                                        border_color: Color::GRAY.into(),
                                        ..default()
                                    },
                                    TrackedInteraction::default(),
                                    InteractiveBackground {
                                        highlight: Some(Color::rgba(0., 1., 1., 0.8)),
                                        ..default()
                                    },
                                    AnimatedInteraction::<InteractiveBackground> {
                                        tween,
                                        ..default()
                                    },
                                    Draggable::default(),
                                    InputSliderDragHandle { slider: input_id },
                                ))
                                .id();
                        })
                        .id();

                    current_value_node = parent
                        .spawn(TextBundle {
                            style: Style {
                                min_width: Val::Px(50.),
                                overflow: Overflow::clip(),
                                align_self: AlignSelf::Center,
                                margin: UiRect::left(Val::Px(5.)),
                                ..default()
                            },
                            ..default()
                        })
                        .id();
                });
        });

        let config = config.unwrap_or_default();
        let initial_ratio = config.initial_value / (config.max - config.min);

        input.insert(InputSlider {
            ratio: initial_ratio,
            config,
            slider_bar,
            drag_handle,
            current_value_node,
        });

        input
    }
}
