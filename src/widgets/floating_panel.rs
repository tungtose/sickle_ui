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
    scroll_interaction::ScrollAxis,
    ui_builder::{UiBuilder, UiBuilderExt},
    FluxInteraction, FluxInteractionUpdate, TrackedInteraction,
};

use super::prelude::UiScrollViewExt;

const MIN_PANEL_SIZE: Vec2 = Vec2 { x: 150., y: 100. };
const MIN_FLOATING_PANEL_Z_INDEX: usize = 1000;
const PRIORITY_FLOATING_PANEL_Z_INDEX: usize = 10000;

pub struct FloatingPanelPlugin;

impl Plugin for FloatingPanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                index_floating_panel.run_if(panel_added),
                process_panel_config_update,
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
    let max = if let Some(Some(m)) = q_panels.iter().map(|p| p.z_index).max() {
        m
    } else {
        0
    };

    let mut offset = 1;
    for mut panel in &mut q_panels.iter_mut() {
        if panel.z_index.is_none() {
            panel.z_index = Some(MIN_FLOATING_PANEL_Z_INDEX + max + offset);
            offset += 1;
        }
    }
}

fn process_panel_config_update(
    q_panels: Query<
        (Entity, &FloatingPanelConfig, &Children),
        (With<FloatingPanel>, Changed<FloatingPanelConfig>),
    >,
    q_title: Query<&FloatingPanelTitle>,
    q_drag_handle: Query<&FloatingPanelDragHandle>,
    q_resize_handles: Query<&FloatingPanelResizeHandleContainer>,
    mut commands: Commands,
) {
    for (panel_id, config, children) in &q_panels {
        if let Some(title) = config.title.clone() {
            if let Some(title_id) = children.iter().find(|&&child| q_title.get(child).is_ok()) {
                commands.entity(*title_id).despawn_recursive();
            }
            if let Some(drag_handle_id) = children
                .iter()
                .find(|&&child| q_drag_handle.get(child).is_ok())
            {
                commands.entity(*drag_handle_id).despawn_recursive();
            }

            let mut title_id = Entity::PLACEHOLDER;
            commands.entity(panel_id).with_children(|parent| {
                title_id =
                    FloatingPanel::add_panel_title(parent, panel_id, title, config.draggable);
            });
            commands.entity(panel_id).insert_children(0, &[title_id]);
        } else {
            if let Some(text_id) = children.iter().find(|&&child| q_title.get(child).is_ok()) {
                commands.entity(*text_id).despawn_recursive();
            }
            if config.draggable {
                if let None = children
                    .iter()
                    .find(|&&child| q_drag_handle.get(child).is_ok())
                {
                    let mut drag_handle_id = Entity::PLACEHOLDER;
                    commands.entity(panel_id).with_children(|parent| {
                        drag_handle_id = FloatingPanel::add_panel_drag_handle(parent, panel_id);
                    });
                    commands
                        .entity(panel_id)
                        .insert_children(0, &[drag_handle_id]);
                }
            } else {
                if let Some(drag_handle_id) = children
                    .iter()
                    .find(|&&child| q_drag_handle.get(child).is_ok())
                {
                    commands.entity(*drag_handle_id).despawn_recursive();
                }
            }
        }

        if config.resizable {
            if let None = children
                .iter()
                .find(|&&child| q_resize_handles.get(child).is_ok())
            {
                let mut resize_handles_id = Entity::PLACEHOLDER;
                commands.entity(panel_id).with_children(|parent| {
                    resize_handles_id = FloatingPanel::add_resize_handles(parent, panel_id);
                });
                commands
                    .entity(panel_id)
                    .insert_children(0, &[resize_handles_id]);
            }
        } else {
            if let Some(resize_handles_id) = children
                .iter()
                .find(|&&child| q_resize_handles.get(child).is_ok())
            {
                commands.entity(*resize_handles_id).despawn_recursive();
            }
        }
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
    mut q_panels: Query<&mut FloatingPanel>,
) {
    if let Some(_) = q_panels.iter().find(|p| p.priority) {
        return;
    }

    for (draggable, handle) in &q_draggable {
        if draggable.state == DragState::Inactive
            || draggable.state == DragState::MaybeDragged
            || draggable.state == DragState::DragCanceled
        {
            continue;
        }

        let Ok(mut panel) = q_panels.get_mut(handle.panel) else {
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
    q_draggable: Query<
        (
            &Draggable,
            AnyOf<(&FloatingPanelTitle, &FloatingPanelDragHandle)>,
        ),
        Changed<Draggable>,
    >,
    mut q_panels: Query<(Entity, &mut FloatingPanel)>,
) {
    if let Some(_) = q_panels.iter().find(|(_, p)| p.priority) {
        return;
    }

    let max_index = if let Some(Some(m)) = q_panels.iter().map(|(_, p)| p.z_index).max() {
        m
    } else {
        0
    };
    let mut offset = 1;

    for (draggable, (panel_title, drag_handle)) in &q_draggable {
        if draggable.state == DragState::Inactive
            || draggable.state == DragState::MaybeDragged
            || draggable.state == DragState::DragCanceled
        {
            continue;
        }

        let panel_id = if let Some(panel_title) = panel_title {
            panel_title.panel
        } else if let Some(drag_handle) = drag_handle {
            drag_handle.panel
        } else {
            continue;
        };

        let Ok((_, mut panel)) = q_panels.get_mut(panel_id) else {
            continue;
        };

        let Some(diff) = draggable.diff else {
            continue;
        };

        panel.z_index = Some(max_index + offset);
        panel.position += diff;
        offset += 1;
    }

    let mut panel_indices: Vec<(Entity, Option<usize>)> = q_panels
        .iter()
        .map(|(entity, panel)| (entity, panel.z_index))
        .collect();
    panel_indices.sort_by(|(_, a), (_, b)| a.cmp(b));

    for (i, (entity, _)) in panel_indices.iter().enumerate() {
        if let Some((_, mut panel)) = q_panels.iter_mut().find(|(e, _)| e == entity) {
            panel.z_index = Some(MIN_FLOATING_PANEL_Z_INDEX + i + 1);
        };
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

        if panel.priority {
            *z_index = ZIndex::Global(PRIORITY_FLOATING_PANEL_Z_INDEX as i32);
        } else if let Some(index) = panel.z_index {
            *z_index = ZIndex::Global(index as i32);
        }
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
pub struct FloatingPanelResizeHandleContainer {
    panel: Entity,
}

impl Default for FloatingPanelResizeHandleContainer {
    fn default() -> Self {
        Self {
            panel: Entity::PLACEHOLDER,
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

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct FloatingPanelDragHandle {
    panel: Entity,
}

impl Default for FloatingPanelDragHandle {
    fn default() -> Self {
        Self {
            panel: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Component, Clone, Debug, Reflect)]
pub struct FloatingPanelConfig {
    pub title: Option<String>,
    pub draggable: bool,
    pub resizable: bool,
    pub restrict_scroll: Option<ScrollAxis>,
}

impl Default for FloatingPanelConfig {
    fn default() -> Self {
        Self {
            title: None,
            draggable: true,
            resizable: true,
            restrict_scroll: None,
        }
    }
}

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct FloatingPanel {
    pub size: Vec2,
    pub position: Vec2,
    pub z_index: Option<usize>,
    pub priority: bool,
}

impl<'w, 's, 'a> FloatingPanel {
    fn base_bundle(config: FloatingPanelConfig, layout: FloatingPanelLayout) -> impl Bundle {
        (
            NodeBundle {
                style: Style {
                    display: if layout.hidden {
                        Display::None
                    } else {
                        Display::Flex
                    },
                    position_type: PositionType::Absolute,
                    width: Val::Px(layout.size.x),
                    height: Val::Px(layout.size.y),
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
            config,
            FloatingPanel {
                size: layout.size,
                position: layout.position.unwrap_or_default(),
                z_index: None,
                priority: false,
            },
        )
    }

    fn add_panel_title(
        parent: &'a mut ChildBuilder<'w, 's, '_>,
        panel: Entity,
        title: String,
        draggable: bool,
    ) -> Entity {
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

        let mut text_id = Entity::PLACEHOLDER;
        title_node.with_children(|parent| {
            text_id = parent
                .spawn(TextBundle {
                    style: Style {
                        margin: UiRect::px(5., 5., 5., 2.),
                        ..default()
                    },
                    text: Text::from_section(title, TextStyle::default()),
                    ..default()
                })
                .id();
        });

        title_node.id()
    }

    fn add_panel_drag_handle(parent: &'a mut ChildBuilder<'w, 's, '_>, panel: Entity) -> Entity {
        let drag_handle = parent.spawn((
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
            FloatingPanelDragHandle { panel },
            TrackedInteraction::default(),
            Draggable::default(),
        ));

        drag_handle.id()
    }

    fn add_resize_handles(parent: &'a mut ChildBuilder<'w, 's, '_>, panel: Entity) -> Entity {
        let zone_size = 4.;
        let zone_pullback = 2.;

        let mut handles = parent.spawn((
            NodeBundle {
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
            },
            FloatingPanelResizeHandleContainer { panel },
        ));

        handles.with_children(|parent| {
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

        handles.id()
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

#[derive(Debug, Default)]
pub struct FloatingPanelLayout {
    pub size: Vec2,
    pub position: Option<Vec2>,
    pub hidden: bool,
}

pub trait UiFloatingPanelExt<'w, 's> {
    fn floating_panel<'a>(
        &'a mut self,
        config: FloatingPanelConfig,
        layout: FloatingPanelLayout,
        spawn_children: impl FnOnce(&mut UiBuilder),
    ) -> EntityCommands<'w, 's, 'a>;
}

impl<'w, 's> UiFloatingPanelExt<'w, 's> for UiBuilder<'w, 's, '_> {
    fn floating_panel<'a>(
        &'a mut self,
        config: FloatingPanelConfig,
        layout: FloatingPanelLayout,
        spawn_children: impl FnOnce(&mut UiBuilder),
    ) -> EntityCommands<'w, 's, 'a> {
        let mut new_parent = Entity::PLACEHOLDER;
        let restrict_to = config.restrict_scroll;

        if let Some(entity) = self.entity() {
            self.commands().entity(entity).with_children(|parent| {
                new_parent = parent
                    .spawn(FloatingPanel::base_bundle(config, layout))
                    .id();
            });
        } else {
            new_parent = self
                .commands()
                .spawn(FloatingPanel::base_bundle(config, layout))
                .id();
        }

        let mut new_entity = self.commands().entity(new_parent);
        let mut new_builder = new_entity.ui_builder();
        new_builder.scroll_view(restrict_to, spawn_children);

        self.commands().entity(new_parent)
    }
}
