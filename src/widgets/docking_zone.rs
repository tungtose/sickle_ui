use bevy::{prelude::*, ui::UiSystem};

use crate::{
    drag_interaction::{DragState, Draggable, DraggableUpdate},
    resize_interaction::{ResizeDirection, ResizeHandle},
    ui_builder::*,
    ui_style::{SetBackgroundColorExt, SetEntityDisplayExt, SetEntityVisiblityExt, UiStyleExt},
};

use super::prelude::UiContainerExt;

const MIN_DOCKING_ZONE_SIZE: f32 = 50.;

pub struct DockingZonePlugin;

impl Plugin for DockingZonePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (
                preset_docking_zone_flex_layout,
                preset_docking_zone_children_size,
                preset_docking_zone_resize_handles,
                preset_docking_zone_border,
            )
                .chain()
                .in_set(DockingZonePreUpdate)
                .run_if(did_add_or_remove_docking_zone),
        )
        .add_systems(
            Update,
            (
                update_docking_zone_on_resize.after(DraggableUpdate),
                update_docking_zone_style,
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

#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct DockingZonePreUpdate;

fn did_add_or_remove_docking_zone(
    q_added_zones: Query<Entity, Added<DockingZone>>,
    mut q_removed_zones: RemovedComponents<DockingZone>,
) -> bool {
    q_added_zones.iter().count() > 0 || q_removed_zones.read().count() > 0
}

fn preset_docking_zone_flex_layout(
    q_docking_zones: Query<(Entity, &Parent), With<DockingZone>>,
    mut q_docking_zone: Query<&mut DockingZone>,
    q_children: Query<&Children>,
    q_style: Query<&Style>,
) {
    let static_zones: Vec<(Entity, Entity)> = q_docking_zones
        .iter()
        .filter(|(_, parent)| q_docking_zone.get(parent.get()).is_err())
        .map(|(e, p)| (e, p.get()))
        .collect();

    for (docking_zone, parent) in static_zones {
        let Ok(parent_style) = q_style.get(parent) else {
            warn!("No Style found for docking zone parent {:?}!", parent);
            continue;
        };

        let parent_flex_direction = parent_style.flex_direction;
        preset_drop_zone_flex_direction(
            docking_zone,
            &mut q_docking_zone,
            &q_children,
            parent_flex_direction,
        );
    }
}

fn preset_drop_zone_flex_direction(
    docking_zone: Entity,
    q_docking_zone: &mut Query<&mut DockingZone>,
    q_children: &Query<&Children>,
    parent_flex_direction: FlexDirection,
) {
    let mut zone = q_docking_zone.get_mut(docking_zone).unwrap();

    zone.flex_direction = match parent_flex_direction {
        FlexDirection::Row => FlexDirection::Column,
        FlexDirection::Column => FlexDirection::Row,
        FlexDirection::RowReverse => FlexDirection::Column,
        FlexDirection::ColumnReverse => FlexDirection::Row,
    };

    let zone_direction = zone.flex_direction;
    if let Ok(children) = q_children.get(docking_zone) {
        for child in children {
            if q_docking_zone.get(*child).is_ok() {
                preset_drop_zone_flex_direction(*child, q_docking_zone, q_children, zone_direction);
            }
        }
    }
}

fn preset_docking_zone_children_size(
    q_docking_zones: Query<Entity, With<DockingZone>>,
    mut q_docking_zone: Query<&mut DockingZone>,
    q_parents: Query<&Parent>,
) {
    for mut zone in &mut q_docking_zone {
        zone.children_size = 0.;
    }

    for entity in &q_docking_zones {
        let zone = q_docking_zone.get(entity).unwrap();
        let zone_size = zone.min_size;
        let direction = zone.flex_direction;

        for parent in q_parents.iter_ancestors(entity) {
            let Ok(mut parent_zone) = q_docking_zone.get_mut(parent) else {
                continue;
            };

            if parent_zone.flex_direction == direction {
                parent_zone.children_size += zone_size;
            }
        }
    }

    for mut zone in &mut q_docking_zone {
        zone.children_size = zone.children_size.max(zone.min_size);
    }
}

fn preset_docking_zone_border(mut q_docking_zones: Query<(&DockingZone, &mut Style)>) {
    for (zone, mut style) in &mut q_docking_zones {
        match zone.flex_direction {
            FlexDirection::Row => {
                style.border = UiRect::vertical(Val::Px(2.));
            }
            FlexDirection::Column => {
                style.border = UiRect::horizontal(Val::Px(2.));
            }
            _ => (),
        }
    }
}

fn preset_docking_zone_resize_handles(
    q_docking_zone_parents: Query<&Parent, With<DockingZone>>,
    q_children: Query<&Children>,
    q_docking_zones: Query<&DockingZone>,
    q_style: Query<&Style>,
    mut q_resize_handle: Query<&mut DockingZoneResizeHandle>,
    mut commands: Commands,
) {
    let zone_count = q_docking_zone_parents.iter().count();
    let mut handle_visibility: Vec<(Entity, bool)> = Vec::with_capacity(zone_count * 4);
    let mut handle_non_display: Vec<(Entity, bool)> = Vec::with_capacity(zone_count * 4);
    let mut handle_neighbours: Vec<(Entity, Option<Entity>)> = Vec::with_capacity(zone_count * 4);
    let parents: Vec<Entity> =
        q_docking_zone_parents
            .iter()
            .fold(Vec::with_capacity(zone_count), |mut acc, parent| {
                let entity = parent.get();
                if !acc.contains(&entity) {
                    acc.push(entity);
                }

                acc
            });

    for parent in parents {
        let children: Vec<Entity> = q_children.get(parent).unwrap().iter().map(|e| *e).collect();
        let child_count = children.len();

        if child_count == 1 {
            let Ok(zone) = q_docking_zones.get(children[0]) else {
                return;
            };
            // handle_visibility.push((zone.top_handle, false));
            // handle_visibility.push((zone.right_handle, false));
            // handle_visibility.push((zone.bottom_handle, false));
            // handle_visibility.push((zone.left_handle, false));
            handle_non_display.push((zone.top_handle, false));
            handle_non_display.push((zone.right_handle, false));
            handle_non_display.push((zone.bottom_handle, false));
            handle_non_display.push((zone.left_handle, false));
        } else {
            let mut zone_children: Vec<Entity> = Vec::with_capacity(child_count);
            let mut prev_is_zone = true;

            for i in 0..child_count {
                let Ok(style) = q_style.get(children[i]) else {
                    warn!(
                        "Missing Style detected on Node {:?} during docking zone handle update.",
                        children[i]
                    );
                    continue;
                };

                let Ok(zone) = q_docking_zones.get(children[i]) else {
                    if style.position_type == PositionType::Relative {
                        prev_is_zone = false;
                    }
                    continue;
                };

                match zone.flex_direction {
                    FlexDirection::Row => {
                        handle_visibility.push((zone.top_handle, !prev_is_zone));
                        handle_visibility.push((zone.bottom_handle, i != child_count - 1));
                        // handle_visibility.push((zone.right_handle, false));
                        // handle_visibility.push((zone.left_handle, false));
                        handle_non_display.push((zone.top_handle, true));
                        handle_non_display.push((zone.bottom_handle, true));
                        handle_non_display.push((zone.left_handle, false));
                        handle_non_display.push((zone.right_handle, false));
                    }
                    FlexDirection::Column => {
                        handle_visibility.push((zone.left_handle, !prev_is_zone));
                        handle_visibility.push((zone.right_handle, i != child_count - 1));
                        handle_non_display.push((zone.left_handle, true));
                        handle_non_display.push((zone.right_handle, true));
                        handle_non_display.push((zone.top_handle, false));
                        handle_non_display.push((zone.bottom_handle, false));
                        // handle_visibility.push((zone.top_handle, false));
                        // handle_visibility.push((zone.bottom_handle, false));
                    }
                    _ => warn!(
                        "Invalid flex_direction detected on docking zone {:?}",
                        children[i]
                    ),
                }

                prev_is_zone = true;
                zone_children.push(children[i]);
            }

            for i in 0..zone_children.len() {
                let zone = q_docking_zones.get(zone_children[i]).unwrap();
                let Some((prev_handle, next_handle)) = (match zone.flex_direction {
                    FlexDirection::Row => (zone.top_handle, zone.bottom_handle).into(),
                    FlexDirection::Column => (zone.left_handle, zone.right_handle).into(),
                    _ => None,
                }) else {
                    warn!(
                        "Invalid flex_direction detected on docking zone {:?}",
                        zone_children[i]
                    );
                    continue;
                };

                if i == 0 {
                    handle_visibility.push((prev_handle, false));
                }

                if i == zone_children.len() - 1 {
                    handle_visibility.push((next_handle, false));
                }

                handle_neighbours.push((
                    prev_handle,
                    match i > 0 {
                        true => zone_children[i - 1].into(),
                        false => None,
                    },
                ));

                handle_neighbours.push((
                    next_handle,
                    match i < zone_children.len() - 1 {
                        true => zone_children[i + 1].into(),
                        false => None,
                    },
                ));
            }
        }
    }

    for (handle, visible) in handle_visibility {
        commands.entity(handle).style().visibility(match visible {
            true => Visibility::Visible,
            false => Visibility::Hidden,
        });
    }

    for (handle, visible) in handle_non_display {
        commands.entity(handle).style().display(match visible {
            true => Display::Flex,
            false => Display::None,
        });
    }

    for (handle, neighbour) in handle_neighbours {
        let mut handle = q_resize_handle.get_mut(handle).unwrap();
        handle.neighbour = neighbour;
    }
}

fn update_docking_zone_on_resize(
    q_draggable: Query<(&Draggable, &ResizeHandle, &DockingZoneResizeHandle), Changed<Draggable>>,
    mut q_docking_zone: Query<(&mut DockingZone, &Parent)>,
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

        let current_zone_id = handle_ref.docking_zone;
        let neighbour_zone_id = handle_ref.neighbour.unwrap();
        let Ok((current_zone, parent)) = q_docking_zone.get(current_zone_id) else {
            continue;
        };
        let Ok((neighbour_zone, other_parent)) = q_docking_zone.get(neighbour_zone_id) else {
            continue;
        };

        if parent != other_parent {
            warn!(
                "Failed to resize docking zone: Neighbouring zones have different parents: {:?} <-> {:?}",
                parent, other_parent
            );
            continue;
        }

        let size_diff = match current_zone.flex_direction {
            FlexDirection::Row => handle.direction().to_size_diff(diff).y,
            FlexDirection::Column => handle.direction().to_size_diff(diff).x,
            _ => 0.,
        };
        if size_diff == 0. {
            continue;
        }

        let Ok(node) = q_node.get(parent.get()) else {
            warn!(
                "Cannot calculate docking zone pixel size: Entity {:?} has parent without Node!",
                current_zone
            );
            continue;
        };

        let total_size = match current_zone.flex_direction {
            FlexDirection::Row => node.size().y,
            FlexDirection::Column => node.size().x,
            _ => 0.,
        };
        if total_size == 0. {
            continue;
        }

        let current_min_size = current_zone.children_size;
        let current_size = (current_zone.size_percent / 100.) * total_size;
        let mut current_new_size = current_size;
        let neighbour_min_size = neighbour_zone.children_size;
        let neighbour_size = (neighbour_zone.size_percent / 100.) * total_size;
        let mut neighbour_new_size = neighbour_size;

        if size_diff < 0. {
            if current_size + size_diff >= current_min_size {
                current_new_size += size_diff;
                neighbour_new_size -= size_diff;
            } else {
                current_new_size = current_min_size;
                neighbour_new_size += current_size - current_min_size;
            }
        } else if size_diff > 0. {
            if neighbour_size - size_diff >= neighbour_min_size {
                neighbour_new_size -= size_diff;
                current_new_size += size_diff;
            } else {
                neighbour_new_size = neighbour_min_size;
                current_new_size += neighbour_size - neighbour_min_size;
            }
        }

        q_docking_zone
            .get_mut(current_zone_id)
            .unwrap()
            .0
            .size_percent = (current_new_size / total_size) * 100.;

        q_docking_zone
            .get_mut(neighbour_zone_id)
            .unwrap()
            .0
            .size_percent = (neighbour_new_size / total_size) * 100.;
    }
}

fn update_docking_zone_style(
    mut q_docking_zones: Query<(&DockingZone, &mut Style), Changed<DockingZone>>,
) {
    for (zone, mut style) in &mut q_docking_zones {
        style.flex_direction = zone.flex_direction;
        match zone.flex_direction {
            FlexDirection::Row => {
                style.width = Val::Percent(100.);
                style.height = Val::Percent(zone.size_percent);
            }
            FlexDirection::Column => {
                style.width = Val::Percent(zone.size_percent);
                style.height = Val::Percent(100.);
            }
            _ => (),
        }
    }
}

fn should_fit_docking_zones(
    q_changed_nodes: Query<Entity, (With<DockingZone>, Changed<Node>)>,
) -> bool {
    q_changed_nodes.iter().count() > 0
}

fn fit_docking_zones_on_window_resize(
    q_children: Query<&Children>,
    q_node: Query<&Node>,
    q_docking_zone_parents: Query<&Parent, With<DockingZone>>,
    q_non_docking: Query<(&Node, &Style), Without<DockingZone>>,
    mut q_docking_zone: Query<(&mut DockingZone, &Node)>,
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
        let Ok(parent_node) = q_node.get(parent) else {
            warn!("Docking zone parent {:?} doesn't have a Node!", parent);
            continue;
        };

        if parent_node.size() == Vec2::ZERO {
            warn!("Docking zone parent {:?} doesn't have a size!", parent);
            continue;
        }

        let mut non_docking_size = Vec2::ZERO;
        for child in q_children.get(parent).unwrap().iter() {
            if let Ok((node, style)) = q_non_docking.get(*child) {
                if style.position_type == PositionType::Relative {
                    non_docking_size += node.size();
                }
            }
        }

        let mut sum_zone_size = Vec2::ZERO;
        for child in q_children.get(parent).unwrap().iter() {
            if let Ok((_, node)) = q_docking_zone.get(*child) {
                sum_zone_size += node.size();
            };
        }

        for child in q_children.get(parent).unwrap().iter() {
            let Ok((mut docking_zone, zone_node)) = q_docking_zone.get_mut(*child) else {
                continue;
            };

            let total_size = match docking_zone.flex_direction {
                FlexDirection::Row => parent_node.size().y,
                FlexDirection::Column => parent_node.size().x,
                _ => 0.,
            };
            let non_docking_size = match docking_zone.flex_direction {
                FlexDirection::Row => non_docking_size.y,
                FlexDirection::Column => non_docking_size.x,
                _ => 0.,
            };
            let sum_zone_size = match docking_zone.flex_direction {
                FlexDirection::Row => sum_zone_size.y,
                FlexDirection::Column => sum_zone_size.x,
                _ => 0.,
            };

            let docking_size = total_size - non_docking_size;

            if total_size == 0. || sum_zone_size == 0. || docking_size <= 0. {
                continue;
            }

            let multiplier = docking_size / sum_zone_size;
            let own_size = match docking_zone.flex_direction {
                FlexDirection::Row => zone_node.size().y,
                FlexDirection::Column => zone_node.size().x,
                _ => 0.,
            };

            // info!(
            //     "{:?} | own: {}, total_size: {}, docking: {}, mul: {}, {}% -> {}%",
            //     child,
            //     own_size,
            //     total_size,
            //     docking_size,
            //     multiplier,
            //     docking_zone.size_percent,
            //     (own_size.max(docking_zone.children_size) / total_size) * 100. * multiplier
            // );

            docking_zone.size_percent =
                (own_size.max(docking_zone.children_size) / total_size) * 100. * multiplier;
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
    flex_direction: FlexDirection,
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
            flex_direction: Default::default(),
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
    ) -> UiBuilder<'w, 's, 'a>;
}

impl<'w, 's> UiDockingZoneExt<'w, 's> for UiBuilder<'w, 's, '_> {
    fn docking_zone<'a>(
        &'a mut self,
        config: DockingZoneConfig,
        spawn_children: impl FnOnce(&mut UiBuilder),
    ) -> UiBuilder<'w, 's, 'a> {
        let size = config.size.clamp(0., 100.);
        let min_size = config.min_size.max(MIN_DOCKING_ZONE_SIZE);
        let mut left_handle = Entity::PLACEHOLDER;
        let mut right_handle = Entity::PLACEHOLDER;
        let mut top_handle = Entity::PLACEHOLDER;
        let mut bottom_handle = Entity::PLACEHOLDER;

        if self.entity().is_none() {
            warn!("Docking zone as root node is not supported!");
        }

        let docking_zone = self
            .container(DockingZone::frame(), |container| {
                let zone_id = container.id();
                let handle = DockingZoneResizeHandle {
                    docking_zone: zone_id,
                    ..default()
                };

                // let mut commands = container.commands().entity(zone_id);
                // let mut new_builder = commands.ui_builder();
                spawn_children(container);

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
            .entity_commands()
            .insert(DockingZone {
                size_percent: size,
                min_size,
                top_handle,
                right_handle,
                bottom_handle,
                left_handle,
                ..default()
            })
            .style()
            .background_color(config.background_color)
            .id();

        self.commands().ui_builder(docking_zone.into())
    }
}
