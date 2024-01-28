use bevy::{
    ecs::system::EntityCommands,
    prelude::*,
    window::{PrimaryWindow, WindowResized},
};
use sickle_math::ease::Ease;

use super::prelude::{LabelConfig, UiContainerExt, UiLabelExt};
use super::prelude::{SetLabelTextExt, UiScrollViewExt};
use crate::{
    animated_interaction::{AnimatedInteraction, AnimationConfig},
    drag_interaction::{DragState, Draggable, DraggableUpdate},
    interactions::InteractiveBackground,
    scroll_interaction::ScrollAxis,
    ui_builder::UiBuilder,
    ui_commands::SetEntityDisplayExt,
    FluxInteraction, FluxInteractionUpdate, TrackedInteraction,
};

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
                process_panel_config_update.before(FluxInteractionUpdate),
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
            panel.z_index = (MIN_FLOATING_PANEL_Z_INDEX + max + offset).into();
            offset += 1;
        }
    }
}

fn process_panel_config_update(
    q_panels: Query<(&FloatingPanel, &FloatingPanelConfig), Changed<FloatingPanelConfig>>,
    mut commands: Commands,
) {
    for (panel, config) in &q_panels {
        commands
            .entity(panel.title_container)
            .set_display(match config.title.is_some() {
                true => Display::Flex,
                false => Display::None,
            });

        if let Some(title) = config.title.clone() {
            commands.entity(panel.title).set_label_text(title);
            if config.draggable {
                commands
                    .entity(panel.title_container)
                    .try_insert((TrackedInteraction::default(), Draggable::default()));
            } else {
                commands
                    .entity(panel.title_container)
                    .remove::<(TrackedInteraction, Draggable)>();
            }
        } else {
            commands
                .entity(panel.drag_handle)
                .set_display(match config.draggable {
                    true => Display::Flex,
                    false => Display::None,
                });
        }

        commands
            .entity(panel.resize_handles)
            .set_display(match config.resizable {
                true => Display::Flex,
                false => Display::None,
            });
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
                    new_cursor = handle.direction.cursor().into()
                }
            }
            FluxInteraction::Pressed => {
                new_cursor = handle.direction.cursor().into();
                *locked = true;
            }
            FluxInteraction::Released => {
                *locked = false;
                if new_cursor.is_none() {
                    new_cursor = CursorIcon::Default.into();
                }
            }
            FluxInteraction::PressCanceled => {
                *locked = false;
                if new_cursor.is_none() {
                    new_cursor = CursorIcon::Default.into();
                }
            }
            FluxInteraction::PointerLeave => {
                if !*locked && new_cursor.is_none() {
                    new_cursor = CursorIcon::Default.into();
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
            panel.z_index = (MIN_FLOATING_PANEL_Z_INDEX + i + 1).into();
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

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct FloatingPanel {
    size: Vec2,
    position: Vec2,
    z_index: Option<usize>,
    title_container: Entity,
    title: Entity,
    drag_handle: Entity,
    resize_handles: Entity,
    content: Entity,
    pub priority: bool,
}

impl Default for FloatingPanel {
    fn default() -> Self {
        Self {
            size: Default::default(),
            position: Default::default(),
            z_index: Default::default(),
            title_container: Entity::PLACEHOLDER,
            title: Entity::PLACEHOLDER,
            drag_handle: Entity::PLACEHOLDER,
            resize_handles: Entity::PLACEHOLDER,
            content: Entity::PLACEHOLDER,
            priority: Default::default(),
        }
    }
}

impl<'w, 's, 'a> FloatingPanel {
    fn base_tween() -> AnimationConfig {
        AnimationConfig {
            duration: 0.1,
            easing: Ease::OutExpo,
            ..default()
        }
    }
    fn resize_zone_size() -> f32 {
        4.
    }
    fn resize_zone_pullback() -> f32 {
        2.
    }

    fn frame() -> impl Bundle {
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                border: UiRect::all(Val::Px(2.)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Start,
                ..default()
            },
            border_color: Color::BLACK.into(),
            background_color: Color::GRAY.into(),
            focus_policy: bevy::ui::FocusPolicy::Block,
            ..default()
        }
    }

    fn title_container() -> impl Bundle {
        ButtonBundle {
            style: Style {
                border: UiRect::right(Val::Px(2.)),
                ..default()
            },
            border_color: Color::BLACK.into(),
            background_color: Color::DARK_GRAY.into(),
            ..default()
        }
    }

    fn drag_handle() -> impl Bundle {
        (
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
            TrackedInteraction::default(),
            Draggable::default(),
        )
    }

    fn resize_handle_container() -> impl Bundle {
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                margin: UiRect::all(Val::Px(-FloatingPanel::resize_zone_pullback())),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            z_index: ZIndex::Local(10),
            ..default()
        }
    }

    fn resize_handle(panel: Entity, direction: ResizeDirection) -> impl Bundle {
        let zone_size = FloatingPanel::resize_zone_size();

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
        (
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
                highlight: Color::rgb(0., 0.5, 1.).into(),
                ..default()
            },
            AnimatedInteraction::<InteractiveBackground> {
                tween: FloatingPanel::base_tween(),
                ..default()
            },
            Draggable::default(),
        )
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
        let restrict_to = config.restrict_scroll;

        let mut resize_handles = Entity::PLACEHOLDER;
        let mut title_container = Entity::PLACEHOLDER;
        let mut title = Entity::PLACEHOLDER;
        let mut drag_handle = Entity::PLACEHOLDER;
        let mut frame = self.container(FloatingPanel::frame(), |container| {
            let panel_id = container.id().unwrap();

            let title_text = if let Some(text) = config.title.clone() {
                text
            } else {
                "".into()
            };

            resize_handles = container
                .container(
                    (
                        FloatingPanel::resize_handle_container(),
                        FloatingPanelResizeHandleContainer { panel: panel_id },
                    ),
                    |resize_container| {
                        resize_container.container(
                            NodeBundle {
                                style: Style {
                                    width: Val::Percent(100.),
                                    height: Val::Px(FloatingPanel::resize_zone_size()),
                                    ..default()
                                },
                                ..default()
                            },
                            |top_row| {
                                top_row.spawn(FloatingPanel::resize_handle(
                                    panel_id,
                                    ResizeDirection::NorthWest,
                                ));
                                top_row.spawn(FloatingPanel::resize_handle(
                                    panel_id,
                                    ResizeDirection::North,
                                ));
                                top_row.spawn(FloatingPanel::resize_handle(
                                    panel_id,
                                    ResizeDirection::NorthEast,
                                ));
                            },
                        );
                        resize_container.container(
                            NodeBundle {
                                style: Style {
                                    width: Val::Percent(100.),
                                    height: Val::Percent(100.),
                                    justify_content: JustifyContent::SpaceBetween,
                                    ..default()
                                },
                                ..default()
                            },
                            |middle_row| {
                                middle_row.spawn(FloatingPanel::resize_handle(
                                    panel_id,
                                    ResizeDirection::West,
                                ));
                                middle_row.spawn(FloatingPanel::resize_handle(
                                    panel_id,
                                    ResizeDirection::East,
                                ));
                            },
                        );
                        resize_container.container(
                            NodeBundle {
                                style: Style {
                                    width: Val::Percent(100.),
                                    height: Val::Px(FloatingPanel::resize_zone_size()),
                                    ..default()
                                },
                                ..default()
                            },
                            |bottom_row| {
                                bottom_row.spawn(FloatingPanel::resize_handle(
                                    panel_id,
                                    ResizeDirection::SouthWest,
                                ));
                                bottom_row.spawn(FloatingPanel::resize_handle(
                                    panel_id,
                                    ResizeDirection::South,
                                ));
                                bottom_row.spawn(FloatingPanel::resize_handle(
                                    panel_id,
                                    ResizeDirection::SouthEast,
                                ));
                            },
                        );
                    },
                )
                .set_display(match config.resizable {
                    true => Display::Flex,
                    false => Display::None,
                })
                .id();

            title_container = container
                .container(
                    (
                        FloatingPanel::title_container(),
                        FloatingPanelTitle { panel: panel_id },
                    ),
                    |container| {
                        title = container
                            .label(LabelConfig {
                                label: title_text,
                                margin: UiRect::px(5., 5., 5., 2.),
                                color: Color::WHITE,
                                ..default()
                            })
                            .id();
                    },
                )
                .set_display(match config.title.is_some() {
                    true => Display::Flex,
                    false => Display::None,
                })
                .id();

            drag_handle = container
                .spawn((
                    FloatingPanel::drag_handle(),
                    FloatingPanelDragHandle { panel: panel_id },
                ))
                .set_display(match config.title.is_some() {
                    true => Display::None,
                    false => Display::Flex,
                })
                .id();

            if layout.hidden {
                container
                    .entity_commands()
                    .unwrap()
                    .set_display(Display::None);
            }

            container.scroll_view(restrict_to, spawn_children);
        });

        frame.insert((
            config,
            FloatingPanel {
                size: layout.size,
                position: layout.position.unwrap_or_default(),
                z_index: None,
                priority: false,
                title_container,
                title,
                drag_handle,
                resize_handles,
                ..default()
            },
        ));

        frame
    }
}
