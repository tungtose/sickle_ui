use bevy::{
    ecs::system::EntityCommands,
    input::mouse::MouseScrollUnit,
    // input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
    ui::FocusPolicy,
};
use sickle_math::{ease::Ease, lerp::Lerp};

use crate::{
    animated_interaction::{AnimatedInteraction, AnimationConfig},
    drag_interaction::{DragState, Draggable, DraggableUpdate},
    interactions::InteractiveBackground,
    scroll_interaction::{ScrollAxis, Scrollable, ScrollableUpdate},
    TrackedInteraction,
};

pub struct InputSliderPlugin;

impl Plugin for InputSliderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_slider_on_scroll.after(ScrollableUpdate),
                update_slider_on_drag.after(DraggableUpdate),
                update_slider_handle,
                update_slider_readout,
            )
                .chain(),
        );
    }
}

// TODO: Remove hardcoded theme
// TODO: Add input for value (w/ read/write flags)

fn update_slider_on_scroll(
    q_scrollables: Query<
        (AnyOf<(&SliderBar, &SliderDragHandle)>, &Scrollable),
        Changed<Scrollable>,
    >,
    mut q_slider: Query<&mut Slider>,
) {
    for ((slider_bar, handle), scrollable) in &q_scrollables {
        let Some((axis, diff, unit)) = scrollable.last_change() else {
            continue;
        };
        if axis == ScrollAxis::Horizontal {
            continue;
        }

        let slider_id = if let Some(slider_bar) = slider_bar {
            slider_bar.slider
        } else if let Some(handle) = handle {
            handle.slider
        } else {
            continue;
        };

        let Ok(mut slider) = q_slider.get_mut(slider_id) else {
            continue;
        };

        let offset = match unit {
            MouseScrollUnit::Line => -diff * 5.,
            MouseScrollUnit::Pixel => -diff,
        };

        let fraction = offset / 100.;
        slider.ratio = (slider.ratio + fraction).clamp(0., 1.);
    }
}

fn update_slider_on_drag(
    q_draggable: Query<(&Draggable, &SliderDragHandle, &Node), Changed<Draggable>>,
    q_node: Query<&Node>,
    mut q_slider: Query<&mut Slider>,
) {
    for (draggable, handle, node) in &q_draggable {
        let Ok(mut slider) = q_slider.get_mut(handle.slider) else {
            continue;
        };

        if draggable.state == DragState::Inactive || draggable.state == DragState::MaybeDragged {
            continue;
        }

        if draggable.state == DragState::DragCanceled {
            if let Some(base_ratio) = slider.base_ratio {
                slider.ratio = base_ratio;
                continue;
            }
        }

        if draggable.state == DragState::DragStart {
            slider.base_ratio = Some(slider.ratio);
        }

        let Ok(slider_bar) = q_node.get(slider.slider_bar) else {
            continue;
        };
        let Some(diff) = draggable.diff else {
            continue;
        };

        let axis = &slider.config.axis;
        let fraction = match axis {
            SliderAxis::Horizontal => {
                let width = slider_bar.size().x - node.size().x;
                if diff.x == 0. || width == 0. {
                    continue;
                }
                diff.x / width
            }
            SliderAxis::Vertical => {
                let height = slider_bar.size().y - node.size().y;
                if diff.y == 0. || height == 0. {
                    continue;
                }
                -diff.y / height
            }
        };

        slider.ratio = (slider.ratio + fraction).clamp(0., 1.);
    }
}

fn update_slider_handle(
    q_slider: Query<&Slider, Or<(Changed<Slider>, Changed<Node>)>>,
    q_node: Query<&Node>,
    mut q_hadle_style: Query<(&Node, &mut Style), With<SliderDragHandle>>,
) {
    for slider in &q_slider {
        let Ok(slider_bar) = q_node.get(slider.slider_bar) else {
            continue;
        };
        let Ok((node, mut style)) = q_hadle_style.get_mut(slider.drag_handle) else {
            continue;
        };

        let axis = &slider.config.axis;
        match axis {
            SliderAxis::Horizontal => {
                let width = slider_bar.size().x - node.size().x;
                let handle_position = width * slider.ratio;
                if style.left != Val::Px(handle_position) {
                    style.left = Val::Px(handle_position);
                }
            }
            SliderAxis::Vertical => {
                let height = slider_bar.size().y - node.size().y;
                let handle_position = height * (1. - slider.ratio);
                if style.top != Val::Px(handle_position) {
                    style.top = Val::Px(handle_position);
                }
            }
        }
    }
}

fn update_slider_readout(
    q_slider: Query<&Slider, Changed<Slider>>,
    mut q_style: Query<&mut Style>,
    mut q_text: Query<&mut Text>,
) {
    for slider in &q_slider {
        let Ok(mut text) = q_text.get_mut(slider.readout_target) else {
            continue;
        };

        let Ok(mut style) = q_style.get_mut(slider.readout_target) else {
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

#[derive(Debug, Default, Eq, PartialEq, Reflect)]
pub enum SliderAxis {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Component, Debug, Reflect)]
pub struct SliderConfig {
    min: f32,
    max: f32,
    initial_value: f32,
    show_current: bool,
    axis: SliderAxis,
}

impl SliderConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from(
        min: f32,
        max: f32,
        initial_value: f32,
        show_current: bool,
        axis: SliderAxis,
    ) -> Self {
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
            axis,
        }
    }

    pub fn horizontal(min: f32, max: f32, initial_value: f32, show_current: bool) -> Self {
        Self::from(
            min,
            max,
            initial_value,
            show_current,
            SliderAxis::Horizontal,
        )
    }

    pub fn vertical(min: f32, max: f32, initial_value: f32, show_current: bool) -> Self {
        Self::from(min, max, initial_value, show_current, SliderAxis::Vertical)
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
            axis: Default::default(),
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct Slider {
    pub ratio: f32,
    pub config: SliderConfig,
    slider_bar: Entity,
    drag_handle: Entity,
    readout_target: Entity,
    base_ratio: Option<f32>,
}

impl Default for Slider {
    fn default() -> Self {
        Self {
            ratio: Default::default(),
            config: Default::default(),
            slider_bar: Entity::PLACEHOLDER,
            drag_handle: Entity::PLACEHOLDER,
            readout_target: Entity::PLACEHOLDER,
            base_ratio: None,
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct SliderDragHandle {
    pub slider: Entity,
}

impl Default for SliderDragHandle {
    fn default() -> Self {
        Self {
            slider: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct SliderBar {
    pub slider: Entity,
}

impl Default for SliderBar {
    fn default() -> Self {
        Self {
            slider: Entity::PLACEHOLDER,
        }
    }
}

impl<'w, 's, 'a> Slider {
    pub fn value(&self) -> f32 {
        self.config.min.lerp(self.config.max, self.ratio)
    }

    pub fn spawn(
        parent: &'a mut ChildBuilder<'w, 's, '_>,
        label: Option<String>,
        config: Option<SliderConfig>,
    ) -> EntityCommands<'w, 's, 'a> {
        let config = config.unwrap_or_default();
        if config.axis == SliderAxis::Horizontal {
            Self::horizontal(parent, label, config)
        } else {
            Self::vertical(parent, label, config)
        }
    }

    fn horizontal(
        parent: &'a mut ChildBuilder<'w, 's, '_>,
        label: Option<String>,
        config: SliderConfig,
    ) -> EntityCommands<'w, 's, 'a> {
        let tween = AnimationConfig {
            duration: 0.1,
            easing: Ease::OutExpo,
            ..default()
        };

        let mut input = parent.spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Px(20.),
                    justify_content: JustifyContent::Start,
                    align_items: AlignItems::Center,
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
        let mut readout_target: Entity = Entity::PLACEHOLDER;
        input.with_children(|parent| {
            if let Some(label) = label {
                parent.spawn(TextBundle {
                    style: Style {
                        margin: UiRect::px(5., 10., 0., 0.),
                        ..default()
                    },
                    text: Text::from_section(
                        label,
                        TextStyle {
                            color: Color::BLACK,
                            font_size: 14.,
                            ..default()
                        },
                    ),
                    focus_policy: FocusPolicy::Pass,
                    ..default()
                });
            }

            slider_bar = parent
                .spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Percent(100.),
                            ..default()
                        },
                        ..default()
                    },
                    SliderBar { slider: input_id },
                    Interaction::default(),
                    Scrollable::default(),
                ))
                .with_children(|parent| {
                    parent
                        .spawn((NodeBundle {
                            style: Style {
                                width: Val::Percent(100.),
                                height: Val::Px(4.),
                                margin: UiRect::vertical(Val::Px(8.)),
                                border: UiRect::px(1., 1., 0., 1.),
                                ..default()
                            },
                            background_color: Color::DARK_GRAY.into(),
                            border_color: Color::GRAY.into(),
                            ..default()
                        },))
                        .with_children(|parent| {
                            drag_handle = parent
                                .spawn((
                                    ButtonBundle {
                                        style: Style {
                                            width: Val::Px(20.),
                                            height: Val::Px(20.),
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
                                    SliderDragHandle { slider: input_id },
                                    Draggable::default(),
                                    Scrollable::default(),
                                ))
                                .id();
                        });
                })
                .id();

            readout_target = parent
                .spawn(TextBundle {
                    style: Style {
                        min_width: Val::Px(50.),
                        margin: UiRect::left(Val::Px(5.)),
                        ..default()
                    },
                    ..default()
                })
                .id();
        });

        let initial_ratio = config.initial_value / (config.max - config.min);

        input.insert(Slider {
            ratio: initial_ratio,
            config,
            slider_bar,
            drag_handle,
            readout_target,
            base_ratio: None,
        });

        input
    }

    fn vertical(
        parent: &'a mut ChildBuilder<'w, 's, '_>,
        label: Option<String>,
        config: SliderConfig,
    ) -> EntityCommands<'w, 's, 'a> {
        let tween = AnimationConfig {
            duration: 0.1,
            easing: Ease::OutExpo,
            ..default()
        };

        let mut input = parent.spawn((
            NodeBundle {
                style: Style {
                    height: Val::Percent(100.),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    margin: UiRect::all(Val::Px(5.)),
                    flex_direction: FlexDirection::Column,
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
            current_value_node = parent
                .spawn(TextBundle {
                    style: Style {
                        margin: UiRect::px(5., 5., 5., 0.),
                        ..default()
                    },
                    focus_policy: FocusPolicy::Pass,
                    ..default()
                })
                .id();

            slider_bar = parent
                .spawn((
                    NodeBundle {
                        style: Style {
                            height: Val::Percent(100.),
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                        ..default()
                    },
                    SliderBar { slider: input_id },
                    Interaction::default(),
                    Scrollable::default(),
                ))
                .with_children(|parent| {
                    parent
                        .spawn((NodeBundle {
                            style: Style {
                                width: Val::Px(4.),
                                height: Val::Percent(100.),
                                flex_direction: FlexDirection::Column,
                                margin: UiRect::horizontal(Val::Px(8.)),
                                border: UiRect::px(1., 1., 0., 1.),
                                ..default()
                            },
                            background_color: Color::DARK_GRAY.into(),
                            border_color: Color::GRAY.into(),
                            ..default()
                        },))
                        .with_children(|parent| {
                            drag_handle = parent
                                .spawn((
                                    ButtonBundle {
                                        style: Style {
                                            width: Val::Px(20.),
                                            height: Val::Px(20.),
                                            margin: UiRect::left(Val::Px(-8.)),
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
                                    SliderDragHandle { slider: input_id },
                                    Draggable::default(),
                                    Scrollable::default(),
                                ))
                                .id();
                        });
                })
                .id();

            if let Some(label) = label {
                parent.spawn(TextBundle {
                    style: Style {
                        margin: UiRect::px(5., 5., 0., 5.),
                        ..default()
                    },
                    text: Text::from_section(
                        label,
                        TextStyle {
                            color: Color::BLACK,
                            font_size: 14.,
                            ..default()
                        },
                    ),
                    focus_policy: FocusPolicy::Pass,
                    ..default()
                });
            }
        });

        let initial_ratio = config.initial_value / (config.max - config.min);

        input.insert(Slider {
            ratio: initial_ratio,
            config,
            slider_bar,
            drag_handle,
            readout_target: current_value_node,
            base_ratio: None,
        });

        input
    }
}
