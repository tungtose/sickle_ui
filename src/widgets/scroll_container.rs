use bevy::{
    ecs::system::EntityCommands,
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
};
use sickle_math::ease::Ease;

use crate::{
    animated_interaction::{AnimatedInteraction, AnimationConfig},
    drag_interaction::{DragState, Draggable},
    interactions::InteractiveBackground,
    TrackedInteraction,
};

pub struct ScrollContainerPlugin;

impl Plugin for ScrollContainerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                move_contents_to_viewport,
                update_scroll_container_on_content_change,
                process_scroll_event,
                update_scroll_on_drag,
                update_scroll_offset,
                update_scroll_container_layout,
            )
                .chain(),
        );
    }
}

fn move_contents_to_viewport(
    q_to_move: Query<(Entity, &MoveToViewport), Added<MoveToViewport>>,
    mut q_scroll: Query<&mut ScrollContainer>,
    mut commands: Commands,
) {
    for (entity, to_move) in &q_to_move {
        let mut container = q_scroll.get_mut(to_move.scroll_container).unwrap();
        container.content_container = entity;
        commands
            .entity(entity)
            .set_parent(to_move.viewport)
            .remove::<MoveToViewport>();
    }
}

fn update_scroll_container_on_content_change(
    q_content: Query<&ScrollContainerContent, Changed<Node>>,
    mut q_scroll_container: Query<&mut ScrollContainer>,
) {
    for content in &q_content {
        let Ok(mut container) = q_scroll_container.get_mut(content.container) else {
            continue;
        };

        // Touch for change
        container.scroll_offset = container.scroll_offset;
    }
}

fn process_scroll_event(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    keys: Res<Input<KeyCode>>,
    q_scrollables: Query<(
        AnyOf<(&ScrollContainerViewport, &ScrollBar, &ScrollBarHandle)>,
        &Interaction,
    )>,
    mut q_scroll_container: Query<&mut ScrollContainer>,
) {
    for mouse_wheel_event in mouse_wheel_events.read() {
        for ((viewport, scroll_bar, bar_handle), interaction) in &q_scrollables {
            if *interaction != Interaction::Hovered {
                continue;
            }

            let mut axis = ScrollAxis::Vertical;
            let mut offset = if mouse_wheel_event.x != 0. {
                -mouse_wheel_event.x
            } else {
                -mouse_wheel_event.y
            };

            if mouse_wheel_event.x > 0.
                || keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight])
            {
                axis = ScrollAxis::Horizontal;
            }

            offset = match mouse_wheel_event.unit {
                MouseScrollUnit::Line => offset * 20.,
                MouseScrollUnit::Pixel => offset,
            };

            let scroll_container_id: Entity;
            if let Some(bar_handle) = bar_handle {
                scroll_container_id = bar_handle.container;
            } else if let Some(scroll_bar) = scroll_bar {
                scroll_container_id = scroll_bar.container;
            } else if let Some(viewport) = viewport {
                scroll_container_id = viewport.container;
            } else {
                continue;
            }

            let Ok(mut scroll_container) = q_scroll_container.get_mut(scroll_container_id) else {
                continue;
            };

            match axis {
                ScrollAxis::Horizontal => {
                    scroll_container.scroll_offset =
                        scroll_container.scroll_offset + Vec2 { x: offset, y: 0. };
                }
                ScrollAxis::Vertical => {
                    scroll_container.scroll_offset =
                        scroll_container.scroll_offset + Vec2 { x: 0., y: offset };
                }
            }
        }
    }
}

fn update_scroll_on_drag(
    q_draggable: Query<(Entity, &Draggable, &ScrollBarHandle), Changed<Draggable>>,
    q_node: Query<&Node>,
    mut q_scroll_container: Query<&mut ScrollContainer>,
) {
    for (entity, draggable, bar_handle) in &q_draggable {
        if draggable.state == DragState::Inactive
            || draggable.state == DragState::MaybeDragged
            || draggable.state == DragState::DragCanceled
        {
            continue;
        }

        let Ok(mut scroll_container) = q_scroll_container.get_mut(bar_handle.container) else {
            continue;
        };
        let Some(diff) = draggable.diff else {
            continue;
        };

        let Ok(bar_node) = q_node.get(entity) else {
            continue;
        };
        let bar_size = match bar_handle.axis {
            ScrollAxis::Horizontal => bar_node.size().x,
            ScrollAxis::Vertical => bar_node.size().y,
        };

        let Ok(content_node) = q_node.get(scroll_container.content_container) else {
            continue;
        };
        let content_size = match bar_handle.axis {
            ScrollAxis::Horizontal => content_node.size().x,
            ScrollAxis::Vertical => content_node.size().y,
        };

        let Ok(container_node) = q_node.get(bar_handle.container) else {
            continue;
        };
        let container_size = match bar_handle.axis {
            ScrollAxis::Horizontal => container_node.size().x,
            ScrollAxis::Vertical => container_node.size().y,
        };

        let overflow = content_size - container_size;
        if overflow <= 0. {
            continue;
        }

        let remaining_space = container_size - bar_size;
        let ratio = overflow / remaining_space;
        let diff = match bar_handle.axis {
            ScrollAxis::Horizontal => diff.x,
            ScrollAxis::Vertical => diff.y,
        } * ratio;

        scroll_container.scroll_offset += match bar_handle.axis {
            ScrollAxis::Horizontal => Vec2 { x: diff, y: 0. },
            ScrollAxis::Vertical => Vec2 { x: 0., y: diff },
        };
    }
}

fn update_scroll_offset(
    mut q_container: Query<(Entity, &mut ScrollContainer), Changed<ScrollContainer>>,
    q_node: Query<&Node>,
) {
    for (entity, mut container) in &mut q_container {
        let Ok(container_node) = q_node.get(entity) else {
            continue;
        };

        let container_width = container_node.size().x;
        let container_height = container_node.size().y;
        if container_width == 0. || container_height == 0. {
            continue;
        }

        let Ok(content_node) = q_node.get(container.content_container) else {
            continue;
        };

        let content_width = content_node.size().x;
        let content_height = content_node.size().y;

        let overflow_x = content_width - container_width;
        let scroll_offset_x = if overflow_x > 0. {
            container.scroll_offset.x.clamp(0., overflow_x)
        } else {
            container.scroll_offset.x
        };
        let overflow_y = content_height - container_height;
        let scroll_offset_y = if overflow_y > 0. {
            container.scroll_offset.y.clamp(0., overflow_y)
        } else {
            container.scroll_offset.y
        };

        container.scroll_offset = Vec2 {
            x: scroll_offset_x,
            y: scroll_offset_y,
        };
    }
}

fn update_scroll_container_layout(
    q_container: Query<(Entity, &ScrollContainer), Or<(Changed<ScrollContainer>, Changed<Node>)>>,
    mut q_node: Query<&Node>,
    mut q_style: Query<&mut Style>,
) {
    for (entity, container) in &q_container {
        let Ok(container_node) = q_node.get(entity) else {
            continue;
        };

        let container_width = container_node.size().x;
        let container_height = container_node.size().y;
        if container_width == 0. || container_height == 0. {
            continue;
        }

        let Ok(content_node) = q_node.get_mut(container.content_container) else {
            continue;
        };
        let Ok(mut content_style) = q_style.get_mut(container.content_container) else {
            continue;
        };

        let content_width = content_node.size().x;
        let content_height = content_node.size().y;

        let overflow_x = content_width - container_width;
        let overflow_y = content_height - container_height;

        // Update content scroll
        if content_height > container_height {
            let scroll_offset_y = container.scroll_offset.y.clamp(0., overflow_y);
            content_style.top = Val::Px(-scroll_offset_y);
        } else {
            content_style.top = Val::Px(0.);
        }
        if content_width > container_width {
            let scroll_offset_x = container.scroll_offset.x.clamp(0., overflow_x);
            content_style.left = Val::Px(-scroll_offset_x);
        } else {
            content_style.left = Val::Px(0.);
        }

        // Update vertical scroll bar
        let Ok(mut vertical_bar_style) = q_style.get_mut(container.vertical_scroll_bar) else {
            continue;
        };
        if container_height >= content_height {
            vertical_bar_style.display = Display::None;
        } else {
            vertical_bar_style.display = Display::Flex;

            let Ok(mut handle_style) = q_style.get_mut(container.vertical_scroll_bar_handle) else {
                continue;
            };
            let scroll_offset_y = container.scroll_offset.y.clamp(0., overflow_y);
            let visible_ratio = (container_height / content_height).clamp(0., 1.);
            let bar_height = (visible_ratio * container_height).clamp(5., container_height);
            let remaining_space = container_height - bar_height;
            let bar_offset = (scroll_offset_y / overflow_y) * remaining_space;

            handle_style.height = Val::Px(bar_height);
            handle_style.top = Val::Px(bar_offset);
        }

        // Update horizontal scroll bar
        let Ok(mut horizontal_bar_style) = q_style.get_mut(container.horizontal_scroll_bar) else {
            continue;
        };
        if container_width >= content_width {
            horizontal_bar_style.display = Display::None;
        } else {
            horizontal_bar_style.display = Display::Flex;

            let Ok(mut handle_style) = q_style.get_mut(container.horizontal_scroll_bar_handle)
            else {
                continue;
            };
            let scroll_offset_x = container.scroll_offset.x.clamp(0., overflow_x);
            let visible_ratio = (container_width / content_width).clamp(0., 1.);
            let bar_width = (visible_ratio * container_width).clamp(5., container_width);
            let remaining_space = container_width - bar_width;
            let bar_offset = (scroll_offset_x / overflow_x) * remaining_space;

            handle_style.width = Val::Px(bar_width);
            handle_style.left = Val::Px(bar_offset);
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Reflect)]
pub enum ScrollAxis {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct ScrollBarHandle {
    axis: ScrollAxis,
    container: Entity,
}

impl Default for ScrollBarHandle {
    fn default() -> Self {
        Self {
            axis: Default::default(),
            container: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct ScrollBar {
    axis: ScrollAxis,
    container: Entity,
    handle: Entity,
}

impl Default for ScrollBar {
    fn default() -> Self {
        Self {
            axis: Default::default(),
            container: Entity::PLACEHOLDER,
            handle: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct ScrollContainerContent {
    container: Entity,
}

impl Default for ScrollContainerContent {
    fn default() -> Self {
        Self {
            container: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct ScrollContainerViewport {
    container: Entity,
}

impl Default for ScrollContainerViewport {
    fn default() -> Self {
        Self {
            container: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[component(storage = "SparseSet")]
#[reflect(Component)]
struct MoveToViewport {
    scroll_container: Entity,
    viewport: Entity,
}

impl Default for MoveToViewport {
    fn default() -> Self {
        Self {
            scroll_container: Entity::PLACEHOLDER,
            viewport: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct ScrollContainer {
    viewport: Entity,
    content_container: Entity,
    horizontal_scroll_bar: Entity,
    horizontal_scroll_bar_handle: Entity,
    vertical_scroll_bar: Entity,
    vertical_scroll_bar_handle: Entity,
    scroll_offset: Vec2,
}

impl Default for ScrollContainer {
    fn default() -> Self {
        Self {
            viewport: Entity::PLACEHOLDER,
            content_container: Entity::PLACEHOLDER,
            horizontal_scroll_bar: Entity::PLACEHOLDER,
            horizontal_scroll_bar_handle: Entity::PLACEHOLDER,
            vertical_scroll_bar: Entity::PLACEHOLDER,
            vertical_scroll_bar_handle: Entity::PLACEHOLDER,
            scroll_offset: Vec2::ZERO,
        }
    }
}

impl<'w, 's, 'a> ScrollContainer {
    pub fn spawn(parent: &'a mut ChildBuilder<'w, 's, '_>) -> EntityCommands<'w, 's, 'a> {
        let mut viewport_id = Entity::PLACEHOLDER;
        let mut horizontal_scroll_id = Entity::PLACEHOLDER;
        let mut horizontal_scroll_handle_id = Entity::PLACEHOLDER;
        let mut vertical_scroll_id = Entity::PLACEHOLDER;
        let mut vertical_scroll_handle_id = Entity::PLACEHOLDER;

        let mut container = parent.spawn((NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                ..default()
            },
            ..default()
        },));

        let scroll_container_id = container.id();

        container.with_children(|parent| {
            viewport_id = parent
                .spawn((
                    NodeBundle {
                        style: Style {
                            position_type: PositionType::Absolute,
                            height: Val::Percent(100.),
                            width: Val::Percent(100.),
                            overflow: Overflow::clip(),
                            ..default()
                        },
                        background_color: Color::DARK_GRAY.into(),
                        ..default()
                    },
                    ScrollContainerViewport {
                        container: scroll_container_id,
                    },
                    Interaction::default(),
                ))
                .id();

            parent
                .spawn(NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        width: Val::Percent(100.),
                        height: Val::Percent(100.),
                        justify_content: JustifyContent::End,
                        align_content: AlignContent::Stretch,
                        ..default()
                    },
                    z_index: ZIndex::Local(1),
                    ..default()
                })
                .with_children(|parent| {
                    (horizontal_scroll_id, horizontal_scroll_handle_id) =
                        ScrollContainer::spawn_scroll_bar(
                            parent,
                            ScrollAxis::Horizontal,
                            scroll_container_id,
                        );
                    (vertical_scroll_id, vertical_scroll_handle_id) =
                        ScrollContainer::spawn_scroll_bar(
                            parent,
                            ScrollAxis::Vertical,
                            scroll_container_id,
                        );
                });
        });

        container.insert((ScrollContainer {
            viewport: viewport_id,
            horizontal_scroll_bar: horizontal_scroll_id,
            horizontal_scroll_bar_handle: horizontal_scroll_handle_id,
            vertical_scroll_bar: vertical_scroll_id,
            vertical_scroll_bar_handle: vertical_scroll_handle_id,
            ..default()
        },));

        let inner_container = parent.spawn((
            NodeBundle {
                style: Style {
                    justify_self: JustifySelf::Start,
                    align_self: AlignSelf::Start,
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::px(0., 12., 0., 12.),
                    ..default()
                },
                ..default()
            },
            MoveToViewport {
                scroll_container: scroll_container_id,
                viewport: viewport_id,
            },
            ScrollContainerContent {
                container: scroll_container_id,
            },
        ));

        inner_container
    }

    fn spawn_scroll_bar(
        parent: &'a mut ChildBuilder<'w, 's, '_>,
        axis: ScrollAxis,
        container: Entity,
    ) -> (Entity, Entity) {
        let tween = AnimationConfig {
            duration: 0.1,
            easing: Ease::OutExpo,
            ..default()
        };

        let mut scroll_bar = parent.spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                width: match axis {
                    ScrollAxis::Horizontal => Val::Percent(100.),
                    ScrollAxis::Vertical => Val::Px(12.),
                },
                height: match axis {
                    ScrollAxis::Horizontal => Val::Px(12.),
                    ScrollAxis::Vertical => Val::Percent(100.),
                },
                flex_direction: match axis {
                    ScrollAxis::Horizontal => FlexDirection::Row,
                    ScrollAxis::Vertical => FlexDirection::Column,
                },
                align_self: AlignSelf::End,
                justify_content: JustifyContent::Start,
                ..default()
            },
            background_color: Color::GRAY.into(),
            ..default()
        });

        let mut handle_id = Entity::PLACEHOLDER;
        scroll_bar.with_children(|parent| {
            handle_id = parent
                .spawn((
                    ButtonBundle {
                        style: Style {
                            width: match axis {
                                ScrollAxis::Horizontal => Val::Auto,
                                ScrollAxis::Vertical => Val::Percent(100.),
                            },
                            height: match axis {
                                ScrollAxis::Horizontal => Val::Percent(100.),
                                ScrollAxis::Vertical => Val::Auto,
                            },
                            ..default()
                        },
                        background_color: Color::rgba(0., 1., 1., 0.4).into(),
                        ..default()
                    },
                    ScrollBarHandle { axis, container },
                    TrackedInteraction::default(),
                    InteractiveBackground {
                        highlight: Some(Color::rgba(0., 1., 1., 0.8)),
                        ..default()
                    },
                    AnimatedInteraction::<InteractiveBackground> { tween, ..default() },
                    Draggable::default(),
                ))
                .id();
        });

        scroll_bar.insert(ScrollBar {
            axis,
            container,
            handle: handle_id,
        });

        (scroll_bar.id(), handle_id)
    }
}
