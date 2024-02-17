use bevy::{ecs::system::EntityCommands, prelude::*, ui::UiSystem, window::WindowResized};

use crate::{
    drag_interaction::{DragState, Draggable, DraggableUpdate},
    resize_interaction::{ResizeDirection, ResizeHandle},
    ui_builder::*,
    ui_style::{SetBackgroundColorExt, SetEntityVisiblityExt, UiStyleExt},
};

use super::prelude::UiContainerExt;

const MIN_DOCKING_ZONE_SIZE: f32 = 50.;

pub struct DockingZonePlugin;

impl Plugin for DockingZonePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            update_docking_zone_resize_handles.run_if(did_add_or_remove_docking_zone),
        )
        .add_systems(
            Update,
            (
                update_docking_zone_on_resize.after(DraggableUpdate),
                update_docking_zone_size,
            )
                .chain(),
        )
        .add_systems(
            PostUpdate,
            fit_docking_zones_on_window_resize
                .run_if(should_fit_docking_zones)
                .after(UiSystem::Layout),
        );
    }
}

fn did_add_or_remove_docking_zone(
    q_added_zones: Query<Entity, Added<DockingZone>>,
    mut q_removed_zones: RemovedComponents<DockingZone>,
) -> bool {
    q_added_zones.iter().count() > 0 || q_removed_zones.read().count() > 0
}

fn _update_docking_zone_flex_layout() {}

fn update_docking_zone_resize_handles(
    q_docking_zone_parents: Query<&Parent, With<DockingZone>>,
    q_children: Query<&Children>,
    q_docking_zones: Query<&DockingZone>,
    q_style: Query<&Style>,
    mut q_resize_handle: Query<&mut DockingZoneResizeHandle>,
    mut commands: Commands,
) {
    let parents: Vec<Entity> = q_docking_zone_parents.iter().fold(
        Vec::with_capacity(q_docking_zone_parents.iter().count()),
        |mut acc, parent| {
            let entity = parent.get();
            if !acc.contains(&entity) {
                acc.push(entity);
            }

            acc
        },
    );

    for parent in parents {
        let children: Vec<Entity> = q_children.get(parent).unwrap().iter().map(|e| *e).collect();
        let child_count = children.len();

        if child_count == 1 {
            let Ok(zone) = q_docking_zones.get(children[0]) else {
                return;
            };
            commands
                .entity(zone.left_handle)
                .style()
                .visibility(Visibility::Hidden);
            commands
                .entity(zone.right_handle)
                .style()
                .visibility(Visibility::Hidden);
        } else {
            let mut zone_children: Vec<Entity> = Vec::with_capacity(child_count);
            let mut prev_is_zone = true;

            for i in 0..child_count {
                let Ok(zone) = q_docking_zones.get(children[i]) else {
                    if let Ok(style) = q_style.get(children[i]) {
                        if style.position_type == PositionType::Relative {
                            prev_is_zone = false;
                        }
                    }
                    continue;
                };

                commands
                    .entity(zone.left_handle)
                    .style()
                    .visibility(match prev_is_zone {
                        true => Visibility::Hidden,
                        false => Visibility::Visible,
                    });

                commands
                    .entity(zone.right_handle)
                    .style()
                    .visibility(match i == child_count - 1 {
                        true => Visibility::Hidden,
                        false => Visibility::Visible,
                    });

                prev_is_zone = true;
                zone_children.push(children[i]);
            }

            for i in 0..zone_children.len() {
                let zone = q_docking_zones.get(zone_children[i]).unwrap();
                let left_handle = zone.left_handle;
                let right_handle = zone.right_handle;

                let mut left_handle = q_resize_handle.get_mut(left_handle).unwrap();
                left_handle.neighbour = if i > 0 {
                    zone_children[i - 1].into()
                } else {
                    None
                };
                left_handle.docking_zone = zone_children[i].into();

                let mut right_handle = q_resize_handle.get_mut(right_handle).unwrap();
                right_handle.docking_zone = zone_children[i].into();
                right_handle.neighbour = if i < zone_children.len() - 1 {
                    zone_children[i + 1].into()
                } else {
                    None
                };
            }
        }
    }
}

// TODO: Consider children min_width for constraints
fn update_docking_zone_on_resize(
    q_draggable: Query<(&Draggable, &ResizeHandle, &DockingZoneResizeHandle), Changed<Draggable>>,
    mut q_flexi_columns: Query<(&mut DockingZone, &Parent)>,
    q_node: Query<&Node>,
) {
    for (draggable, handle, handle_ref) in &q_draggable {
        if handle_ref.neighbour.is_none() {
            continue;
        }

        if draggable.state == DragState::Inactive
            || draggable.state == DragState::MaybeDragged
            || draggable.state == DragState::DragCanceled
        {
            continue;
        }

        let Some(diff) = draggable.diff else {
            continue;
        };

        let size_diff = handle.direction().to_size_diff(diff).x;
        if size_diff == 0. {
            continue;
        }

        let current_column_id = handle_ref.docking_zone;
        let neighbour_column_id = handle_ref.neighbour.unwrap();

        let Ok((current_column, parent)) = q_flexi_columns.get(current_column_id) else {
            continue;
        };
        let Ok((neighbour_column, other_parent)) = q_flexi_columns.get(neighbour_column_id) else {
            continue;
        };

        if parent != other_parent {
            warn!(
                "Failed to resize flexi column: Columns have different parents: {:?} <-> {:?}",
                parent, other_parent
            );
            continue;
        }

        let Ok(node) = q_node.get(parent.get()) else {
            warn!(
                "Cannot calculate FlexiColumn pixel width: Entity {:?} has parent without Node!",
                current_column_id
            );
            continue;
        };

        let total_width = node.size().x;

        if total_width == 0. {
            continue;
        }

        let current_min_width = current_column.min_size;
        let current_width = (current_column.size_percent / 100.) * total_width;
        let mut current_new_width = current_width;
        let neighbour_min_width = neighbour_column.min_size;
        let neighbour_width = (neighbour_column.size_percent / 100.) * total_width;
        let mut neighbour_new_width = neighbour_width;

        if size_diff < 0. {
            if current_width + size_diff >= current_min_width {
                current_new_width += size_diff;
                neighbour_new_width -= size_diff;
            } else {
                current_new_width = current_min_width;
                neighbour_new_width += current_width - current_min_width;
            }
        } else if size_diff > 0. {
            if neighbour_width - size_diff >= neighbour_min_width {
                neighbour_new_width -= size_diff;
                current_new_width += size_diff;
            } else {
                neighbour_new_width = neighbour_min_width;
                current_new_width += neighbour_width - neighbour_min_width;
            }
        }

        q_flexi_columns
            .get_mut(current_column_id)
            .unwrap()
            .0
            .size_percent = (current_new_width / total_width) * 100.;

        q_flexi_columns
            .get_mut(neighbour_column_id)
            .unwrap()
            .0
            .size_percent = (neighbour_new_width / total_width) * 100.;
    }
}

fn update_docking_zone_size(
    mut q_flexi_columns: Query<(&DockingZone, &mut Style), Changed<DockingZone>>,
) {
    for (config, mut style) in &mut q_flexi_columns {
        style.width = Val::Percent(config.size_percent);
    }
}

fn should_fit_docking_zones(
    q_added_columns: Query<Entity, Added<DockingZone>>,
    mut q_removed_columns: RemovedComponents<DockingZone>,
    mut e_resize: EventReader<WindowResized>,
) -> bool {
    q_added_columns.iter().count() > 0
        || q_removed_columns.read().count() > 0
        || e_resize.read().count() > 0
}

fn fit_docking_zones_on_window_resize(
    q_children: Query<&Children>,
    q_node: Query<&Node>,
    q_flexi_column_parents: Query<&Parent, With<DockingZone>>,
    q_non_flexi: Query<(&Node, &Style), Without<DockingZone>>,
    mut q_flexi_column: Query<(&mut DockingZone, &Node)>,
) {
    let parents: Vec<Entity> = q_flexi_column_parents.iter().fold(
        Vec::with_capacity(q_flexi_column_parents.iter().count()),
        |mut acc, parent| {
            let entity = parent.get();
            if !acc.contains(&entity) {
                acc.push(entity);
            }

            acc
        },
    );

    for parent in parents {
        let Ok(node) = q_node.get(parent) else {
            warn!("Flexi column parent {:?} doesn't have a Node!", parent);
            continue;
        };

        if node.size().x == 0. {
            warn!("Flexi column parent {:?} doesn't have a size!", parent);
            continue;
        }

        let mut non_flexi_width = 0.;
        for child in q_children.get(parent).unwrap().iter() {
            if let Ok((node, style)) = q_non_flexi.get(*child) {
                if style.position_type == PositionType::Relative {
                    non_flexi_width += node.size().x;
                }
            }
        }

        let mut sum_flexi_width = 0.;
        for child in q_children.get(parent).unwrap().iter() {
            if let Ok((_, node)) = q_flexi_column.get(*child) {
                sum_flexi_width += node.size().x;
            };
        }

        let total_width = node.size().x;
        for child in q_children.get(parent).unwrap().iter() {
            let Ok((mut flexi_column, node)) = q_flexi_column.get_mut(*child) else {
                return;
            };

            let flexi_width = total_width - non_flexi_width;
            let multiplier = flexi_width / sum_flexi_width;

            flexi_column.size_percent =
                (node.size().x.max(flexi_column.min_size) / flexi_width) * 100. * multiplier;
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
pub struct DockingZoneResizeHandle {
    pub docking_zone: Entity,
    pub neighbour: Option<Entity>,
}

impl Default for DockingZoneResizeHandle {
    fn default() -> Self {
        Self {
            docking_zone: Entity::PLACEHOLDER,
            neighbour: Default::default(),
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct DockingZone {
    size_percent: f32,
    min_size: f32,
    children_size: f32,
    top_handle: Entity,
    right_handle: Entity,
    bottom_handle: Entity,
    left_handle: Entity,
}

impl Default for DockingZone {
    fn default() -> Self {
        Self {
            size_percent: Default::default(),
            min_size: MIN_DOCKING_ZONE_SIZE,
            children_size: Default::default(),
            top_handle: Entity::PLACEHOLDER,
            right_handle: Entity::PLACEHOLDER,
            bottom_handle: Entity::PLACEHOLDER,
            left_handle: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Debug, Default)]
pub struct DockingZoneConfig {
    pub size: f32,
    pub min_size: f32,
    pub background_color: Color,
}

impl DockingZone {
    fn container() -> impl Bundle {
        NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                ..default()
            },
            ..default()
        }
    }

    fn frame() -> impl Bundle {
        NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                ..default()
            },
            border_color: Color::BLACK.into(),
            ..default()
        }
    }

    fn vertical_handles_container() -> impl Bundle {
        NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            ..default()
        }
    }
}

pub trait UiDockingZoneExt<'w, 's> {
    fn docking_zone<'a>(
        &'a mut self,
        config: DockingZoneConfig,
        spawn_children: impl FnOnce(&mut UiBuilder),
    ) -> EntityCommands<'w, 's, 'a>;
}

impl<'w, 's> UiDockingZoneExt<'w, 's> for UiBuilder<'w, 's, '_> {
    fn docking_zone<'a>(
        &'a mut self,
        config: DockingZoneConfig,
        spawn_children: impl FnOnce(&mut UiBuilder),
    ) -> EntityCommands<'w, 's, 'a> {
        let size = config.size.clamp(0., 100.);
        let min_size = config.min_size.max(MIN_DOCKING_ZONE_SIZE);
        let mut left_handle = Entity::PLACEHOLDER;
        let mut right_handle = Entity::PLACEHOLDER;
        let mut top_handle = Entity::PLACEHOLDER;
        let mut bottom_handle = Entity::PLACEHOLDER;

        if self.id().is_none() {
            warn!("Docking zone as root node is not supported! An additional node has been injected as container.");
        }

        let mut root = match self.id().is_none() {
            true => self.spawn(DockingZone::container()),
            false => self.entity_commands().unwrap(),
        };

        let docking_zone = root
            .ui_builder()
            .container(DockingZone::frame(), |container| {
                let handle = DockingZoneResizeHandle {
                    docking_zone: container.id().unwrap(),
                    ..default()
                };

                container
                    .container(DockingZone::container(), spawn_children)
                    .style()
                    .background_color(config.background_color);

                container.container(
                    ResizeHandle::resize_handle_container(),
                    |resize_container| {
                        top_handle = resize_container
                            .spawn((ResizeHandle::resize_handle(ResizeDirection::North), handle))
                            .id();
                        resize_container.container(
                            DockingZone::vertical_handles_container(),
                            |middle_row| {
                                left_handle = middle_row
                                    .spawn((
                                        ResizeHandle::resize_handle(ResizeDirection::West),
                                        handle,
                                    ))
                                    .id();
                                right_handle = middle_row
                                    .spawn((
                                        ResizeHandle::resize_handle(ResizeDirection::East),
                                        handle,
                                    ))
                                    .id();
                            },
                        );
                        bottom_handle = resize_container
                            .spawn((ResizeHandle::resize_handle(ResizeDirection::South), handle))
                            .id();
                    },
                );
            })
            .insert(DockingZone {
                size_percent: size,
                min_size,
                top_handle,
                right_handle,
                bottom_handle,
                left_handle,
                ..default()
            })
            .id();

        self.commands().entity(docking_zone)
    }
}
