use bevy::window::PrimaryWindow;
use bevy::{prelude::*, window::WindowResized};

use super::prelude::{LabelConfig, UiContainerExt, UiLabelExt};
use super::prelude::{SetLabelTextExt, UiScrollViewExt};
use crate::resize_interaction::ResizeHandle;
use crate::ui_style::{SetEntityVisiblityExt, UiStyleExt};
use crate::{
    drag_interaction::{DragState, Draggable, DraggableUpdate},
    resize_interaction::ResizeDirection,
    scroll_interaction::ScrollAxis,
    ui_builder::UiBuilder,
    FluxInteractionUpdate, TrackedInteraction,
};

const MIN_PANEL_SIZE: Vec2 = Vec2 { x: 150., y: 100. };
const MIN_FLOATING_PANEL_Z_INDEX: usize = 1000;
const PRIORITY_FLOATING_PANEL_Z_INDEX: usize = 10000;
const WINDOW_RESIZE_PADDING: f32 = 20.;

pub struct FloatingPanelPlugin;

impl Plugin for FloatingPanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                index_floating_panel.run_if(panel_added),
                process_panel_config_update.before(FluxInteractionUpdate),
                update_panel_size_on_resize.after(DraggableUpdate),
                update_panel_on_title_drag.after(DraggableUpdate),
                handle_window_resize.run_if(window_resized),
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
            .style(panel.title_container)
            .visibility(match config.title.is_some() {
                true => Visibility::Inherited,
                false => Visibility::Hidden,
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
                .style(panel.drag_handle)
                .visibility(match config.draggable {
                    true => Visibility::Inherited,
                    false => Visibility::Hidden,
                });
        }

        commands
            .style(panel.resize_handles)
            .visibility(match config.resizable {
                true => Visibility::Inherited,
                false => Visibility::Hidden,
            });
    }
}

fn update_panel_size_on_resize(
    q_draggable: Query<(&Draggable, &ResizeHandle, &FloatingPanelResizeHandle), Changed<Draggable>>,
    mut q_panels: Query<&mut FloatingPanel>,
) {
    if let Some(_) = q_panels.iter().find(|p| p.priority) {
        return;
    }

    for (draggable, handle, handle_ref) in &q_draggable {
        let Ok(mut panel) = q_panels.get_mut(handle_ref.panel) else {
            continue;
        };

        if draggable.state == DragState::Inactive
            || draggable.state == DragState::MaybeDragged
            || draggable.state == DragState::DragCanceled
        {
            panel.resizing = false;
            continue;
        }

        let Some(diff) = draggable.diff else {
            continue;
        };

        let size_diff = handle.direction().to_size_diff(diff);

        let old_size = panel.size;
        panel.resizing = true;
        panel.size += size_diff;
        if draggable.state == DragState::DragEnd {
            if panel.size.x < MIN_PANEL_SIZE.x {
                panel.size.x = MIN_PANEL_SIZE.x;
            }
            if panel.size.y < MIN_PANEL_SIZE.y {
                panel.size.y = MIN_PANEL_SIZE.y;
            }
        }

        let pos_diff = match handle.direction() {
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

    let mut panel_updated = false;

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

        if panel.resizing {
            continue;
        }

        panel.z_index = Some(max_index + offset);
        panel.position += diff;
        offset += 1;
        panel_updated = true;
    }

    if !panel_updated {
        return;
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

fn window_resized(e_resized: EventReader<WindowResized>) -> bool {
    e_resized.len() > 0
}

// TODO: Use the panel's render window
fn handle_window_resize(
    q_window: Query<&Window, With<PrimaryWindow>>,
    mut q_panels: Query<(&mut FloatingPanel, &Node, &GlobalTransform)>,
) {
    let Ok(window) = q_window.get_single() else {
        return;
    };

    for (mut panel, node, transform) in &mut q_panels {
        let position = transform.translation().truncate() - (node.size() / 2.);

        if position.x > window.width() - WINDOW_RESIZE_PADDING {
            panel.position.x = (panel.position.x - panel.size.x + WINDOW_RESIZE_PADDING).max(0.);
            if position.y > window.height() - panel.size.y {
                let overflow = position.y - (window.height() - panel.size.y);
                panel.position.y = (panel.position.y - overflow).max(0.);
            }
        }
        if position.y > window.height() - WINDOW_RESIZE_PADDING {
            panel.position.y = (panel.position.y - panel.size.y + WINDOW_RESIZE_PADDING).max(0.);

            if position.x > window.width() - panel.size.x {
                let overflow = position.x - (window.width() - panel.size.x);
                panel.position.x = (panel.position.x - overflow).max(0.);
            }
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

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct FloatingPanelResizeHandle {
    panel: Entity,
}

impl Default for FloatingPanelResizeHandle {
    fn default() -> Self {
        Self {
            panel: Entity::PLACEHOLDER,
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
    resizing: bool,
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
            resizing: Default::default(),
            priority: Default::default(),
        }
    }
}

impl FloatingPanel {
    fn frame() -> impl Bundle {
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                border: UiRect::all(Val::Px(2.)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Start,
                overflow: Overflow::clip(),
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
}

#[derive(Debug)]
pub struct FloatingPanelLayout {
    pub size: Vec2,
    pub position: Option<Vec2>,
    pub hidden: bool,
}

impl Default for FloatingPanelLayout {
    fn default() -> Self {
        Self {
            size: Vec2 { x: 300., y: 500. },
            position: Default::default(),
            hidden: Default::default(),
        }
    }
}

impl FloatingPanelLayout {
    pub fn min() -> Self {
        Self {
            size: MIN_PANEL_SIZE,
            ..default()
        }
    }
}

pub trait UiFloatingPanelExt<'w, 's> {
    fn floating_panel<'a>(
        &'a mut self,
        config: FloatingPanelConfig,
        layout: FloatingPanelLayout,
        spawn_children: impl FnOnce(&mut UiBuilder),
    ) -> UiBuilder<'w, 's, 'a>;
}

impl<'w, 's> UiFloatingPanelExt<'w, 's> for UiBuilder<'w, 's, '_> {
    fn floating_panel<'a>(
        &'a mut self,
        config: FloatingPanelConfig,
        layout: FloatingPanelLayout,
        spawn_children: impl FnOnce(&mut UiBuilder),
    ) -> UiBuilder<'w, 's, 'a> {
        let restrict_to = config.restrict_scroll;

        let mut resize_handles = Entity::PLACEHOLDER;
        let mut title_container = Entity::PLACEHOLDER;
        let mut title = Entity::PLACEHOLDER;
        let mut drag_handle = Entity::PLACEHOLDER;
        let mut frame = self.container(FloatingPanel::frame(), |container| {
            let panel = container.id();

            let title_text = if let Some(text) = config.title.clone() {
                text
            } else {
                "".into()
            };

            resize_handles = container
                .container(
                    (
                        ResizeHandle::resize_handle_container(),
                        FloatingPanelResizeHandleContainer { panel },
                    ),
                    |resize_container| {
                        resize_container.container(
                            NodeBundle {
                                style: Style {
                                    width: Val::Percent(100.),
                                    height: Val::Px(ResizeHandle::resize_zone_size()),
                                    ..default()
                                },
                                ..default()
                            },
                            |top_row| {
                                top_row.spawn((
                                    ResizeHandle::resize_handle(ResizeDirection::NorthWest),
                                    FloatingPanelResizeHandle { panel },
                                ));
                                top_row.spawn((
                                    ResizeHandle::resize_handle(ResizeDirection::North),
                                    FloatingPanelResizeHandle { panel },
                                ));
                                top_row.spawn((
                                    ResizeHandle::resize_handle(ResizeDirection::NorthEast),
                                    FloatingPanelResizeHandle { panel },
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
                                middle_row.spawn((
                                    ResizeHandle::resize_handle(ResizeDirection::West),
                                    FloatingPanelResizeHandle { panel },
                                ));
                                middle_row.spawn((
                                    ResizeHandle::resize_handle(ResizeDirection::East),
                                    FloatingPanelResizeHandle { panel },
                                ));
                            },
                        );
                        resize_container.container(
                            NodeBundle {
                                style: Style {
                                    width: Val::Percent(100.),
                                    height: Val::Px(ResizeHandle::resize_zone_size()),
                                    ..default()
                                },
                                ..default()
                            },
                            |bottom_row| {
                                bottom_row.spawn((
                                    ResizeHandle::resize_handle(ResizeDirection::SouthWest),
                                    FloatingPanelResizeHandle { panel },
                                ));
                                bottom_row.spawn((
                                    ResizeHandle::resize_handle(ResizeDirection::South),
                                    FloatingPanelResizeHandle { panel },
                                ));
                                bottom_row.spawn((
                                    ResizeHandle::resize_handle(ResizeDirection::SouthEast),
                                    FloatingPanelResizeHandle { panel },
                                ));
                            },
                        );
                    },
                )
                .style()
                .visibility(match config.resizable {
                    true => Visibility::Inherited,
                    false => Visibility::Hidden,
                })
                .id();

            title_container = container
                .container(
                    (
                        FloatingPanel::title_container(),
                        FloatingPanelTitle { panel },
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
                .style()
                .visibility(match config.title.is_some() {
                    true => Visibility::Inherited,
                    false => Visibility::Hidden,
                })
                .id();

            drag_handle = container
                .spawn((
                    FloatingPanel::drag_handle(),
                    FloatingPanelDragHandle { panel },
                ))
                .style()
                .visibility(match config.title.is_some() {
                    true => Visibility::Hidden,
                    false => Visibility::Inherited,
                })
                .id();

            if layout.hidden {
                container.style().visibility(Visibility::Hidden);
            }

            container.scroll_view(restrict_to, spawn_children);
        });

        frame.insert((
            config,
            FloatingPanel {
                size: layout.size.max(MIN_PANEL_SIZE),
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
