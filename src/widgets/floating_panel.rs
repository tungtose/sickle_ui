use bevy::{
    ecs::system::EntityCommands,
    prelude::*,
    window::{PrimaryWindow, WindowResized},
};
use sickle_math::ease::Ease;

use crate::{
    animated_interaction::{AnimatedInteraction, AnimationConfig},
    drag_interaction::{DragState, Draggable, DraggableUpdate},
    interactions::InteractiveBackground,
    FluxInteraction, FluxInteractionUpdate, TrackedInteraction, scroll_interaction::ScrollAxis,
};

use super::{hierarchy::MoveToParent, scroll_view::ScrollView};

const MIN_PANEL_SIZE: Vec2 = Vec2 { x: 150., y: 100. };

pub struct FloatingPanelPlugin;

impl Plugin for FloatingPanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                index_floating_panel.run_if(panel_added),
                update_cursor_on_resize_handles.after(FluxInteractionUpdate),
                update_panel_size_on_resize.after(DraggableUpdate),
                update_panel_on_title_drag.after(DraggableUpdate),
                handle_window_resize,
                update_panel_layout,
            )
                .chain(),
        );
    }
}

fn panel_added(q_panels: Query<Entity, Added<FloatingPanel>>) -> bool {
    q_panels.iter().count() > 0
}

fn index_floating_panel(mut q_panels: Query<&mut FloatingPanel>) {
    for (i, mut panel) in &mut q_panels.iter_mut().enumerate() {
        panel.z_index = i + 1;
    }
}

fn update_cursor_on_resize_handles(
    q_flux: Query<(&FloatingPanelResizeHandle, &FluxInteraction), Changed<FluxInteraction>>,
    mut q_window: Query<&mut Window, With<PrimaryWindow>>,
    mut locked: Local<bool>,
) {
    let Ok(mut window) = q_window.get_single_mut() else {
        return;
    };

    let mut new_cursor: Option<CursorIcon> = None;
    for (handle, flux) in &q_flux {
        match *flux {
            FluxInteraction::PointerEnter => {
                if !*locked {
                    new_cursor = Some(handle.direction.cursor())
                }
            }
            FluxInteraction::Pressed => {
                new_cursor = Some(handle.direction.cursor());
                *locked = true;
            }
            FluxInteraction::Released => {
                *locked = false;
                if new_cursor.is_none() {
                    new_cursor = Some(CursorIcon::Default);
                }
            }
            FluxInteraction::PressCanceled => {
                *locked = false;
                if new_cursor.is_none() {
                    new_cursor = Some(CursorIcon::Default);
                }
            }
            FluxInteraction::PointerLeave => {
                if !*locked && new_cursor.is_none() {
                    new_cursor = Some(CursorIcon::Default);
                }
            }
            _ => (),
        }
    }

    if let Some(new_cursor) = new_cursor {
        window.cursor.icon = new_cursor;
    }
}

fn update_panel_size_on_resize(
    q_draggable: Query<(&Draggable, &FloatingPanelResizeHandle), Changed<Draggable>>,
    mut q_panel: Query<&mut FloatingPanel>,
) {
    for (draggable, handle) in &q_draggable {
        if draggable.state == DragState::Inactive
            || draggable.state == DragState::MaybeDragged
            || draggable.state == DragState::DragCanceled
        {
            continue;
        }

        let Ok(mut panel) = q_panel.get_mut(handle.panel) else {
            continue;
        };
        let Some(diff) = draggable.diff else {
            continue;
        };

        let size_diff = match handle.direction {
            ResizeDirection::North => Vec2 { x: 0., y: -diff.y },
            ResizeDirection::NorthEast => Vec2 {
                x: diff.x,
                y: -diff.y,
            },
            ResizeDirection::East => Vec2 { x: diff.x, y: 0. },
            ResizeDirection::SouthEast => diff,
            ResizeDirection::South => Vec2 { x: 0., y: diff.y },
            ResizeDirection::SouthWest => Vec2 {
                x: -diff.x,
                y: diff.y,
            },
            ResizeDirection::West => Vec2 { x: -diff.x, y: 0. },
            ResizeDirection::NorthWest => Vec2 {
                x: -diff.x,
                y: -diff.y,
            },
        };

        let old_size = panel.size;
        panel.size += size_diff;
        if draggable.state == DragState::DragEnd {
            if panel.size.x < MIN_PANEL_SIZE.x {
                panel.size.x = MIN_PANEL_SIZE.x;
            }
            if panel.size.y < MIN_PANEL_SIZE.y {
                panel.size.y = MIN_PANEL_SIZE.y;
            }
        }

        let pos_diff = match handle.direction {
            ResizeDirection::North => Vec2 {
                x: 0.,
                y: clip_position_change(diff.y, MIN_PANEL_SIZE.y, old_size.y, panel.size.y),
            },
            ResizeDirection::NorthEast => Vec2 {
                x: 0.,
                y: clip_position_change(diff.y, MIN_PANEL_SIZE.y, old_size.y, panel.size.y),
            },
            ResizeDirection::East => Vec2::ZERO,
            ResizeDirection::SouthEast => Vec2::ZERO,
            ResizeDirection::South => Vec2::ZERO,
            ResizeDirection::SouthWest => Vec2 {
                x: clip_position_change(diff.x, MIN_PANEL_SIZE.x, old_size.x, panel.size.x),
                y: 0.,
            },
            ResizeDirection::West => Vec2 {
                x: clip_position_change(diff.x, MIN_PANEL_SIZE.x, old_size.x, panel.size.x),
                y: 0.,
            },
            ResizeDirection::NorthWest => Vec2 {
                x: clip_position_change(diff.x, MIN_PANEL_SIZE.x, old_size.x, panel.size.x),
                y: clip_position_change(diff.y, MIN_PANEL_SIZE.y, old_size.y, panel.size.y),
            },
        };

        panel.position += pos_diff;
    }
}

fn clip_position_change(diff: f32, min: f32, old_size: f32, new_size: f32) -> f32 {
    let mut new_diff = diff;
    if old_size <= min && new_size <= min {
        new_diff = 0.;
    } else if old_size > min && new_size <= min {
        new_diff -= min - new_size;
    } else if old_size < min && new_size >= min {
        new_diff += min - old_size;
    }

    new_diff
}

fn update_panel_on_title_drag(
    q_draggable: Query<(&Draggable, &FloatingPanelTitle), Changed<Draggable>>,
    mut q_panel: Query<&mut FloatingPanel>,
) {
    for (draggable, panel_title) in &q_draggable {
        if draggable.state == DragState::Inactive
            || draggable.state == DragState::MaybeDragged
            || draggable.state == DragState::DragCanceled
        {
            continue;
        }

        let Ok(mut panel) = q_panel.get_mut(panel_title.panel) else {
            continue;
        };
        let Some(diff) = draggable.diff else {
            continue;
        };

        panel.position += diff;
    }
}

fn handle_window_resize(
    events: EventReader<WindowResized>,
    mut q_panels: Query<&mut FloatingPanel>,
) {
    if events.len() > 0 {
        for mut panel in &mut q_panels {
            panel.position = panel.position;
        }
    }
}

fn update_panel_layout(
    mut q_panels: Query<(&FloatingPanel, &mut Style, &mut ZIndex), Changed<FloatingPanel>>,
) {
    for (panel, mut style, mut z_index) in &mut q_panels {
        style.width = Val::Px(panel.size.x.max(MIN_PANEL_SIZE.x));
        style.height = Val::Px(panel.size.y.max(MIN_PANEL_SIZE.y));
        style.left = Val::Px(panel.position.x);
        style.top = Val::Px(panel.position.y);
        *z_index = ZIndex::Local(panel.z_index as i32);
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Reflect)]
pub enum ResizeDirection {
    #[default]
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

impl ResizeDirection {
    fn cursor(&self) -> CursorIcon {
        match self {
            ResizeDirection::North => CursorIcon::NResize,
            ResizeDirection::NorthEast => CursorIcon::NeResize,
            ResizeDirection::East => CursorIcon::EResize,
            ResizeDirection::SouthEast => CursorIcon::SeResize,
            ResizeDirection::South => CursorIcon::SResize,
            ResizeDirection::SouthWest => CursorIcon::SwResize,
            ResizeDirection::West => CursorIcon::WResize,
            ResizeDirection::NorthWest => CursorIcon::NwResize,
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct FloatingPanelResizeHandle {
    panel: Entity,
    direction: ResizeDirection,
}

impl Default for FloatingPanelResizeHandle {
    fn default() -> Self {
        Self {
            panel: Entity::PLACEHOLDER,
            direction: Default::default(),
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct FloatingPanelTitle {
    panel: Entity,
}

impl Default for FloatingPanelTitle {
    fn default() -> Self {
        Self {
            panel: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct FloatingPanel {
    pub size: Vec2,
    pub position: Vec2,
    pub z_index: usize,
}

impl<'w, 's, 'a> FloatingPanel {
    pub fn build(
        parent: &'a mut ChildBuilder<'w, 's, '_>,
        title: Option<String>,
        size: Vec2,
        position: Option<Vec2>,
        draggable: bool,
        resizable: bool,
        restrict_scroll: Option<ScrollAxis>,
    ) -> (Entity, EntityCommands<'w, 's, 'a>) {
        let mut panel = parent.spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    width: Val::Px(size.x),
                    height: Val::Px(size.y),
                    border: UiRect::all(Val::Px(2.)),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Start,
                    ..default()
                },
                border_color: Color::BLACK.into(),
                background_color: Color::GRAY.into(),
                focus_policy: bevy::ui::FocusPolicy::Block,
                ..default()
            },
            FloatingPanel {
                size,
                position: position.unwrap_or_default(),
                z_index: 0,
            },
            MoveToParent { parent: None },
        ));

        let panel_id = panel.id();
        let mut container_id = Entity::PLACEHOLDER;
        panel.with_children(|parent| {
            if resizable {
                FloatingPanel::add_resize_handles(parent, panel_id);
            }

            if let Some(title) = title {
                FloatingPanel::add_panel_title(parent, panel_id, title, draggable);
            } else if draggable {
                FloatingPanel::add_panel_drag_handle(parent, panel_id);
            }

            container_id = parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.),
                        height: Val::Percent(100.),
                        ..default()
                    },
                    ..default()
                })
                .id();
        });

        (
            panel_id,
            ScrollView::spawn_docked(parent, container_id.into(), restrict_scroll),
        )
    }

    fn add_panel_title(
        parent: &'a mut ChildBuilder<'w, 's, '_>,
        panel: Entity,
        title: String,
        draggable: bool,
    ) {
        let mut title_node = parent.spawn((
            ButtonBundle {
                style: Style {
                    border: UiRect::right(Val::Px(2.)),
                    ..default()
                },
                border_color: Color::BLACK.into(),
                background_color: Color::DARK_GRAY.into(),
                ..default()
            },
            FloatingPanelTitle { panel },
        ));

        if draggable {
            title_node.insert((TrackedInteraction::default(), Draggable::default()));
        }

        title_node.with_children(|parent| {
            parent.spawn(TextBundle {
                style: Style {
                    margin: UiRect::px(5., 5., 5., 2.),
                    ..default()
                },
                text: Text::from_section(title, TextStyle::default()),
                ..default()
            });
        });
    }

    fn add_panel_drag_handle(parent: &'a mut ChildBuilder<'w, 's, '_>, panel: Entity) {
        parent.spawn((
            ButtonBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Px(6.),
                    border: UiRect::vertical(Val::Px(2.)),
                    ..default()
                },
                border_color: Color::GRAY.into(),
                background_color: Color::BLACK.into(),
                ..default()
            },
            FloatingPanelTitle { panel },
            TrackedInteraction::default(),
            Draggable::default(),
        ));
    }

    fn add_resize_handles(parent: &'a mut ChildBuilder<'w, 's, '_>, panel: Entity) {
        let zone_size = 4.;
        let zone_pullback = 2.;

        parent
            .spawn(NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    margin: UiRect::all(Val::Px(-zone_pullback)),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                z_index: ZIndex::Local(10),
                ..default()
            })
            .with_children(|parent| {
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            width: Val::Percent(100.),
                            height: Val::Px(zone_size),
                            ..default()
                        },
                        ..default()
                    })
                    .with_children(|parent| {
                        FloatingPanel::add_resize_handle(
                            parent,
                            zone_size,
                            panel,
                            ResizeDirection::NorthWest,
                        );
                        FloatingPanel::add_resize_handle(
                            parent,
                            zone_size,
                            panel,
                            ResizeDirection::North,
                        );
                        FloatingPanel::add_resize_handle(
                            parent,
                            zone_size,
                            panel,
                            ResizeDirection::NorthEast,
                        );
                    });
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            width: Val::Percent(100.),
                            height: Val::Percent(100.),
                            justify_content: JustifyContent::SpaceBetween,
                            ..default()
                        },
                        ..default()
                    })
                    .with_children(|parent| {
                        FloatingPanel::add_resize_handle(
                            parent,
                            zone_size,
                            panel,
                            ResizeDirection::West,
                        );
                        FloatingPanel::add_resize_handle(
                            parent,
                            zone_size,
                            panel,
                            ResizeDirection::East,
                        );
                    });
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            width: Val::Percent(100.),
                            height: Val::Px(zone_size),
                            ..default()
                        },
                        ..default()
                    })
                    .with_children(|parent| {
                        FloatingPanel::add_resize_handle(
                            parent,
                            zone_size,
                            panel,
                            ResizeDirection::SouthWest,
                        );
                        FloatingPanel::add_resize_handle(
                            parent,
                            zone_size,
                            panel,
                            ResizeDirection::South,
                        );
                        FloatingPanel::add_resize_handle(
                            parent,
                            zone_size,
                            panel,
                            ResizeDirection::SouthEast,
                        );
                    });
            });
    }

    fn add_resize_handle(
        parent: &'a mut ChildBuilder<'w, 's, '_>,
        zone_size: f32,
        panel: Entity,
        direction: ResizeDirection,
    ) {
        let tween = AnimationConfig {
            duration: 0.1,
            easing: Ease::OutExpo,
            ..default()
        };

        let (width, height) = match direction {
            ResizeDirection::North => (Val::Percent(100.), Val::Px(zone_size)),
            ResizeDirection::NorthEast => (Val::Px(zone_size), Val::Px(zone_size)),
            ResizeDirection::East => (Val::Px(zone_size), Val::Percent(100.)),
            ResizeDirection::SouthEast => (Val::Px(zone_size), Val::Px(zone_size)),
            ResizeDirection::South => (Val::Percent(100.), Val::Px(zone_size)),
            ResizeDirection::SouthWest => (Val::Px(zone_size), Val::Px(zone_size)),
            ResizeDirection::West => (Val::Px(zone_size), Val::Percent(100.)),
            ResizeDirection::NorthWest => (Val::Px(zone_size), Val::Px(zone_size)),
        };

        parent.spawn((
            NodeBundle {
                style: Style {
                    width,
                    height,
                    ..default()
                },
                focus_policy: bevy::ui::FocusPolicy::Block,
                ..default()
            },
            FloatingPanelResizeHandle { panel, direction },
            Interaction::default(),
            TrackedInteraction::default(),
            InteractiveBackground {
                highlight: Some(Color::rgb(0., 0.5, 1.)),
                ..default()
            },
            AnimatedInteraction::<InteractiveBackground> { tween, ..default() },
            Draggable::default(),
        ));
    }
}
