use bevy::{ecs::system::EntityCommands, input::mouse::MouseScrollUnit, prelude::*};
use sickle_math::ease::Ease;

use crate::{
    animated_interaction::{AnimatedInteraction, AnimationConfig},
    drag_interaction::{DragState, Draggable, DraggableUpdate},
    interactions::InteractiveBackground,
    scroll_interaction::{ScrollAxis, Scrollable, ScrollableUpdate},
    ui_builder::{UiBuilder, UiBuilderExt},
    TrackedInteraction,
};

use super::hierarchy::MoveToParent;

pub struct ScrollViewPlugin;

impl Plugin for ScrollViewPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                move_scrolled_contents_to_viewport,
                update_scroll_view_on_content_change,
                update_scroll_view_on_scroll.after(ScrollableUpdate),
                update_scroll_view_on_drag.after(DraggableUpdate),
                update_scroll_view_offset,
                update_scroll_view_layout,
            )
                .chain(),
        );
    }
}

fn move_scrolled_contents_to_viewport(
    q_to_move: Query<(Entity, &MoveToViewport), Added<MoveToViewport>>,
    mut q_scroll_view: Query<&mut ScrollView>,
    mut commands: Commands,
) {
    for (entity, to_move) in &q_to_move {
        let mut scroll_view = q_scroll_view.get_mut(to_move.scroll_view).unwrap();
        scroll_view.content_container = entity;
        commands
            .entity(entity)
            .set_parent(to_move.viewport)
            .remove::<MoveToViewport>();
    }
}

fn update_scroll_view_on_content_change(
    q_content: Query<&ScrollViewContent, Changed<Node>>,
    mut q_scroll_view: Query<&mut ScrollView>,
) {
    for content in &q_content {
        let Ok(mut container) = q_scroll_view.get_mut(content.scroll_view) else {
            continue;
        };

        // Touch for change
        container.scroll_offset = container.scroll_offset;
    }
}

fn update_scroll_view_on_scroll(
    q_scrollables: Query<
        (
            Entity,
            AnyOf<(&ScrollViewViewport, &ScrollBarHandle, &ScrollThrough)>,
            &Scrollable,
        ),
        Changed<Scrollable>,
    >,
    q_parent: Query<&Parent>,
    mut q_scroll_view: Query<&mut ScrollView>,
) {
    for (entity, (viewport, handle, scroll_through), scrollable) in &q_scrollables {
        let Some((axis, diff, unit)) = scrollable.last_change() else {
            continue;
        };

        let scroll_container_id = if let Some(viewport) = viewport {
            viewport.scroll_view
        } else if let Some(handle) = handle {
            handle.scroll_view
        } else if let Some(_) = scroll_through {
            let mut found = Entity::PLACEHOLDER;
            for parent in q_parent.iter_ancestors(entity) {
                if let Ok(_) = q_scroll_view.get(parent) {
                    found = parent;
                }
            }

            found
        } else {
            continue;
        };

        let Ok(mut scroll_view) = q_scroll_view.get_mut(scroll_container_id) else {
            continue;
        };

        let offset = match axis {
            ScrollAxis::Horizontal => Vec2 { x: diff, y: 0. },
            ScrollAxis::Vertical => Vec2 { x: 0., y: diff },
        };
        let diff = match unit {
            MouseScrollUnit::Line => offset * 20.,
            MouseScrollUnit::Pixel => offset,
        };
        scroll_view.scroll_offset = scroll_view.scroll_offset + diff;
    }
}

fn update_scroll_view_on_drag(
    q_draggable: Query<(Entity, &Draggable, &ScrollBarHandle), Changed<Draggable>>,
    q_node: Query<&Node>,
    mut q_scroll_view: Query<&mut ScrollView>,
) {
    for (entity, draggable, bar_handle) in &q_draggable {
        if draggable.state == DragState::Inactive
            || draggable.state == DragState::MaybeDragged
            || draggable.state == DragState::DragCanceled
        {
            continue;
        }

        let Ok(mut scroll_view) = q_scroll_view.get_mut(bar_handle.scroll_view) else {
            continue;
        };
        let Some(diff) = draggable.diff else {
            continue;
        };
        let Ok(bar_node) = q_node.get(entity) else {
            continue;
        };
        let Ok(content_node) = q_node.get(scroll_view.content_container) else {
            continue;
        };
        let Ok(container_node) = q_node.get(bar_handle.scroll_view) else {
            continue;
        };

        let container_size = match bar_handle.axis {
            ScrollAxis::Horizontal => container_node.size().x,
            ScrollAxis::Vertical => container_node.size().y,
        };
        let content_size = match bar_handle.axis {
            ScrollAxis::Horizontal => content_node.size().x,
            ScrollAxis::Vertical => content_node.size().y,
        };
        let overflow = content_size - container_size;
        if overflow <= 0. {
            continue;
        }

        let bar_size = match bar_handle.axis {
            ScrollAxis::Horizontal => bar_node.size().x,
            ScrollAxis::Vertical => bar_node.size().y,
        };
        let remaining_space = container_size - bar_size;
        let ratio = overflow / remaining_space;
        let diff = match bar_handle.axis {
            ScrollAxis::Horizontal => diff.x,
            ScrollAxis::Vertical => diff.y,
        } * ratio;

        scroll_view.scroll_offset += match bar_handle.axis {
            ScrollAxis::Horizontal => Vec2 { x: diff, y: 0. },
            ScrollAxis::Vertical => Vec2 { x: 0., y: diff },
        };
    }
}

fn update_scroll_view_offset(
    mut q_scroll_view: Query<(Entity, &mut ScrollView), Changed<ScrollView>>,
    q_node: Query<&Node>,
) {
    for (entity, mut scroll_view) in &mut q_scroll_view {
        let Ok(container_node) = q_node.get(entity) else {
            continue;
        };

        let container_width = container_node.size().x;
        let container_height = container_node.size().y;
        if container_width == 0. || container_height == 0. {
            continue;
        }

        let Ok(content_node) = q_node.get(scroll_view.content_container) else {
            continue;
        };

        let content_width = content_node.size().x;
        let content_height = content_node.size().y;

        let overflow_x = content_width - container_width;
        let scroll_offset_x = if overflow_x > 0. {
            scroll_view.scroll_offset.x.clamp(0., overflow_x)
        } else {
            scroll_view.scroll_offset.x
        };
        let overflow_y = content_height - container_height;
        let scroll_offset_y = if overflow_y > 0. {
            scroll_view.scroll_offset.y.clamp(0., overflow_y)
        } else {
            scroll_view.scroll_offset.y
        };

        scroll_view.scroll_offset = Vec2 {
            x: scroll_offset_x,
            y: scroll_offset_y,
        };
    }
}

fn update_scroll_view_layout(
    q_scroll_view: Query<(Entity, &ScrollView), Or<(Changed<ScrollView>, Changed<Node>)>>,
    mut q_node: Query<&Node>,
    mut q_style: Query<&mut Style>,
) {
    for (entity, scroll_view) in &q_scroll_view {
        let Ok(container_node) = q_node.get(entity) else {
            continue;
        };

        let container_width = container_node.size().x;
        let container_height = container_node.size().y;
        if container_width == 0. || container_height == 0. {
            continue;
        }

        let Ok(content_node) = q_node.get_mut(scroll_view.content_container) else {
            continue;
        };
        let Ok(mut content_style) = q_style.get_mut(scroll_view.content_container) else {
            continue;
        };

        let content_width = content_node.size().x;
        let content_height = content_node.size().y;

        let overflow_x = content_width - container_width;
        let overflow_y = content_height - container_height;

        // Update content scroll
        if content_height > container_height {
            let scroll_offset_y = scroll_view.scroll_offset.y.clamp(0., overflow_y);
            content_style.top = Val::Px(-scroll_offset_y);
        } else {
            content_style.top = Val::Px(0.);
        }
        if content_width > container_width {
            let scroll_offset_x = scroll_view.scroll_offset.x.clamp(0., overflow_x);
            content_style.left = Val::Px(-scroll_offset_x);
        } else {
            content_style.left = Val::Px(0.);
        }

        // Update vertical scroll bar
        if let (Some(vertical_scroll_bar), Some(vertical_scroll_bar_handle)) = (
            scroll_view.vertical_scroll_bar,
            scroll_view.vertical_scroll_bar_handle,
        ) {
            if let Ok(mut vertical_bar_style) = q_style.get_mut(vertical_scroll_bar) {
                if container_height >= content_height || container_height <= 5. {
                    vertical_bar_style.display = Display::None;
                } else {
                    vertical_bar_style.display = Display::Flex;

                    if let Ok(mut handle_style) = q_style.get_mut(vertical_scroll_bar_handle) {
                        let scroll_offset_y = scroll_view.scroll_offset.y.clamp(0., overflow_y);
                        let visible_ratio = (container_height / content_height).clamp(0., 1.);
                        let bar_height =
                            (visible_ratio * container_height).clamp(5., container_height);
                        let remaining_space = container_height - bar_height;
                        let bar_offset = (scroll_offset_y / overflow_y) * remaining_space;

                        handle_style.height = Val::Px(bar_height);
                        handle_style.top = Val::Px(bar_offset);
                    };
                }
            };
        }

        if let (Some(horizontal_scroll_bar), Some(horizontal_scroll_bar_handle)) = (
            scroll_view.horizontal_scroll_bar,
            scroll_view.horizontal_scroll_bar_handle,
        ) {
            // Update horizontal scroll bar
            if let Ok(mut horizontal_bar_style) = q_style.get_mut(horizontal_scroll_bar) {
                if container_width >= content_width || container_width <= 5. {
                    horizontal_bar_style.display = Display::None;
                } else {
                    horizontal_bar_style.display = Display::Flex;

                    if let Ok(mut handle_style) = q_style.get_mut(horizontal_scroll_bar_handle) {
                        let scroll_offset_x = scroll_view.scroll_offset.x.clamp(0., overflow_x);
                        let visible_ratio = (container_width / content_width).clamp(0., 1.);
                        let bar_width =
                            (visible_ratio * container_width).clamp(5., container_width);
                        let remaining_space = container_width - bar_width;
                        let bar_offset = (scroll_offset_x / overflow_x) * remaining_space;

                        handle_style.width = Val::Px(bar_width);
                        handle_style.left = Val::Px(bar_offset);
                    };
                }
            };
        }
    }
}

#[derive(Component, Debug)]
pub struct ScrollThrough;

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct ScrollBarHandle {
    axis: ScrollAxis,
    scroll_view: Entity,
}

impl Default for ScrollBarHandle {
    fn default() -> Self {
        Self {
            axis: Default::default(),
            scroll_view: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct ScrollBar {
    axis: ScrollAxis,
    scroll_view: Entity,
    handle: Entity,
}

impl Default for ScrollBar {
    fn default() -> Self {
        Self {
            axis: Default::default(),
            scroll_view: Entity::PLACEHOLDER,
            handle: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct ScrollViewContent {
    scroll_view: Entity,
}

impl Default for ScrollViewContent {
    fn default() -> Self {
        Self {
            scroll_view: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct ScrollViewViewport {
    scroll_view: Entity,
}

impl Default for ScrollViewViewport {
    fn default() -> Self {
        Self {
            scroll_view: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[component(storage = "SparseSet")]
#[reflect(Component)]
struct MoveToViewport {
    scroll_view: Entity,
    viewport: Entity,
}

impl Default for MoveToViewport {
    fn default() -> Self {
        Self {
            scroll_view: Entity::PLACEHOLDER,
            viewport: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct ScrollView {
    viewport: Entity,
    content_container: Entity,
    horizontal_scroll_bar: Option<Entity>,
    horizontal_scroll_bar_handle: Option<Entity>,
    vertical_scroll_bar: Option<Entity>,
    vertical_scroll_bar_handle: Option<Entity>,
    scroll_offset: Vec2,
}

impl Default for ScrollView {
    fn default() -> Self {
        Self {
            viewport: Entity::PLACEHOLDER,
            content_container: Entity::PLACEHOLDER,
            horizontal_scroll_bar: None,
            horizontal_scroll_bar_handle: None,
            vertical_scroll_bar: None,
            vertical_scroll_bar_handle: None,
            scroll_offset: Vec2::ZERO,
        }
    }
}

impl<'w, 's> ScrollView {
    pub fn spawn<'a>(parent: &'a mut ChildBuilder<'w, 's, '_>) -> EntityCommands<'w, 's, 'a> {
        ScrollView::spawn_docked(parent, None, None)
    }

    pub fn horizontal<'a>(parent: &'a mut ChildBuilder<'w, 's, '_>) -> EntityCommands<'w, 's, 'a> {
        ScrollView::spawn_docked(parent, None, Some(ScrollAxis::Horizontal))
    }

    pub fn vertical<'a>(parent: &'a mut ChildBuilder<'w, 's, '_>) -> EntityCommands<'w, 's, 'a> {
        ScrollView::spawn_docked(parent, None, Some(ScrollAxis::Vertical))
    }

    pub fn horizontal_docked<'a>(
        parent: &'a mut ChildBuilder<'w, 's, '_>,
        dock_id: Option<Entity>,
    ) -> EntityCommands<'w, 's, 'a> {
        ScrollView::spawn_docked(parent, dock_id, Some(ScrollAxis::Horizontal))
    }

    pub fn vertical_docked<'a>(
        parent: &'a mut ChildBuilder<'w, 's, '_>,
        dock_id: Option<Entity>,
    ) -> EntityCommands<'w, 's, 'a> {
        ScrollView::spawn_docked(parent, dock_id, Some(ScrollAxis::Vertical))
    }

    pub fn spawn_docked<'a>(
        parent: &'a mut ChildBuilder<'w, 's, '_>,
        dock_id: Option<Entity>,
        restrict_to: Option<ScrollAxis>,
    ) -> EntityCommands<'w, 's, 'a> {
        let mut scroll_view = parent.spawn(ScrollView::shell_bundle());

        if dock_id.is_some() {
            scroll_view.insert(MoveToParent { parent: dock_id });
        }

        let scroll_view_id = scroll_view.id();
        let component = ScrollView::fill_scroll_view(&mut scroll_view, restrict_to);

        scroll_view.insert(component.clone());

        let content_container = parent.spawn((
            ScrollView::content_bundle(scroll_view_id),
            MoveToViewport {
                scroll_view: scroll_view_id,
                viewport: component.viewport,
            },
        ));

        content_container
    }

    fn fill_scroll_view<'a>(
        shell: &'a mut EntityCommands<'w, 's, '_>,
        restrict_to: Option<ScrollAxis>,
    ) -> ScrollView {
        let mut viewport_id = Entity::PLACEHOLDER;
        let mut horizontal_scroll_id = Entity::PLACEHOLDER;
        let mut horizontal_scroll_handle_id = Entity::PLACEHOLDER;
        let mut vertical_scroll_id = Entity::PLACEHOLDER;
        let mut vertical_scroll_handle_id = Entity::PLACEHOLDER;
        let scroll_view_id = shell.id();

        shell.with_children(|parent| {
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
                    ScrollViewViewport {
                        scroll_view: scroll_view_id,
                    },
                    Interaction::default(),
                    Scrollable::default(),
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
                    if let Some(restrict_to) = restrict_to {
                        match restrict_to {
                            ScrollAxis::Horizontal => {
                                (horizontal_scroll_id, horizontal_scroll_handle_id) =
                                    ScrollView::spawn_scroll_bar(
                                        parent,
                                        ScrollAxis::Horizontal,
                                        scroll_view_id,
                                    )
                            }
                            ScrollAxis::Vertical => {
                                (vertical_scroll_id, vertical_scroll_handle_id) =
                                    ScrollView::spawn_scroll_bar(
                                        parent,
                                        ScrollAxis::Vertical,
                                        scroll_view_id,
                                    )
                            }
                        }
                    } else {
                        (horizontal_scroll_id, horizontal_scroll_handle_id) =
                            ScrollView::spawn_scroll_bar(
                                parent,
                                ScrollAxis::Horizontal,
                                scroll_view_id,
                            );
                        (vertical_scroll_id, vertical_scroll_handle_id) =
                            ScrollView::spawn_scroll_bar(
                                parent,
                                ScrollAxis::Vertical,
                                scroll_view_id,
                            );
                    }
                });
        });

        ScrollView {
            viewport: viewport_id,
            horizontal_scroll_bar: horizontal_scroll_id.into(),
            horizontal_scroll_bar_handle: horizontal_scroll_handle_id.into(),
            vertical_scroll_bar: vertical_scroll_id.into(),
            vertical_scroll_bar_handle: vertical_scroll_handle_id.into(),
            ..default()
        }
    }

    fn spawn_scroll_bar<'a>(
        parent: &'a mut ChildBuilder<'w, 's, '_>,
        axis: ScrollAxis,
        scroll_view: Entity,
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
                    ScrollBarHandle { axis, scroll_view },
                    TrackedInteraction::default(),
                    InteractiveBackground {
                        highlight: Some(Color::rgba(0., 1., 1., 0.8)),
                        ..default()
                    },
                    AnimatedInteraction::<InteractiveBackground> { tween, ..default() },
                    Draggable::default(),
                    Scrollable::default(),
                ))
                .id();
        });

        scroll_bar.insert(ScrollBar {
            axis,
            scroll_view,
            handle: handle_id,
        });

        (scroll_bar.id(), handle_id)
    }

    fn shell_bundle() -> impl Bundle {
        NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                ..default()
            },
            ..default()
        }
    }

    fn content_bundle(scroll_view: Entity) -> impl Bundle {
        (
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
            ScrollViewContent { scroll_view },
        )
    }
}

pub trait UiScrollViewExt<'w, 's> {
    fn scroll_view<'a>(
        &'a mut self,
        restrict_to: Option<ScrollAxis>,
        f: impl FnOnce(&mut UiBuilder),
    ) -> EntityCommands<'w, 's, 'a>;
}

impl<'w, 's> UiScrollViewExt<'w, 's> for UiBuilder<'w, 's, '_> {
    fn scroll_view<'a>(
        &'a mut self,
        restrict_to: Option<ScrollAxis>,
        spawn_children: impl FnOnce(&mut UiBuilder),
    ) -> EntityCommands<'w, 's, 'a> {
        let mut scroll_view = Entity::PLACEHOLDER;
        let mut content_container = Entity::PLACEHOLDER;

        if let Some(entity) = self.entity() {
            self.commands().entity(entity).with_children(|parent| {
                scroll_view = parent.spawn(ScrollView::shell_bundle()).id();
            });
        } else {
            scroll_view = self.commands().spawn(ScrollView::shell_bundle()).id();
        }

        let mut scroll_view_commands = self.commands().entity(scroll_view);
        let mut component = ScrollView::fill_scroll_view(&mut scroll_view_commands, restrict_to);

        self.commands()
            .entity(component.viewport)
            .with_children(|parent| {
                content_container = parent.spawn(ScrollView::content_bundle(scroll_view)).id();
            });

        component.content_container = content_container;
        self.commands().entity(scroll_view).insert(component);

        let mut new_entity = self.commands().entity(content_container);
        let mut new_builder = new_entity.ui_builder();
        spawn_children(&mut new_builder);

        self.commands().entity(scroll_view)
    }
}
