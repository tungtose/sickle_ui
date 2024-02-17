use bevy::{
    ecs::system::EntityCommands,
    prelude::*,
    ui::UiSystem,
    window::{PrimaryWindow, WindowResized},
};

use crate::{
    drag_interaction::{DragState, Draggable, DraggableUpdate},
    resize_interaction::{ResizeDirection, ResizeHandle},
    ui_builder::*,
    ui_style::{SetEntityVisiblityExt, UiStyleExt},
};

use super::prelude::{RowConfig, UiContainerExt, UiRowExt};

const MIN_FLEXI_ROW_HEIGHT: f32 = 50.;

pub struct FlexiRowPlugin;

impl Plugin for FlexiRowPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            update_flexi_row_resize_handles.run_if(should_update_resize_handles),
        )
        .add_systems(
            Update,
            (
                update_flexi_row_on_resize.after(DraggableUpdate),
                update_flexi_row_height,
            )
                .chain(),
        )
        .add_systems(
            PostUpdate,
            fit_flexi_rows_on_window_resize
                .run_if(should_fit_flexi_rows)
                .after(UiSystem::Layout),
        );
    }
}

fn should_update_resize_handles(
    q_added_rows: Query<Entity, Added<FlexiRow>>,
    mut q_removed_rows: RemovedComponents<FlexiRow>,
) -> bool {
    q_added_rows.iter().count() > 0 || q_removed_rows.read().count() > 0
}

fn update_flexi_row_resize_handles(
    q_flexi_row_parents: Query<&Parent, With<FlexiRow>>,
    q_children: Query<&Children>,
    q_root_flexi_rows: Query<Entity, (With<FlexiRow>, Without<Parent>)>,
    q_non_flexi_root: Query<Entity, (With<Node>, Without<Parent>)>,
    q_flexi_rows: Query<&FlexiRow>,
    q_style: Query<&Style>,
    mut q_resize_handle: Query<&mut FlexiRowResizeHandle>,
    mut commands: Commands,
) {
    let parents: Vec<Entity> = q_flexi_row_parents.iter().fold(
        Vec::with_capacity(q_flexi_row_parents.iter().count()),
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
        update_flexi_row_handles_on_collection(
            children,
            &q_flexi_rows,
            &q_style,
            &mut q_resize_handle,
            &mut commands,
        );
    }

    if q_root_flexi_rows.iter().count() > 0 {
        let children: Vec<Entity> = q_non_flexi_root.iter().map(|e| e).collect();
        update_flexi_row_handles_on_collection(
            children,
            &q_flexi_rows,
            &q_style,
            &mut q_resize_handle,
            &mut commands,
        );
    }
}

fn update_flexi_row_handles_on_collection(
    children: Vec<Entity>,
    q_flexi_rows: &Query<&FlexiRow>,
    q_style: &Query<&Style>,
    q_resize_handle: &mut Query<&mut FlexiRowResizeHandle>,
    commands: &mut Commands,
) {
    let child_count = children.len();

    if child_count == 1 {
        let Ok(flexi) = q_flexi_rows.get(children[0]) else {
            return;
        };

        commands
            .entity(flexi.top_handle)
            .style()
            .visibility(Visibility::Hidden);
        commands
            .entity(flexi.bottom_handle)
            .style()
            .visibility(Visibility::Hidden);
    } else {
        let mut flexi_children: Vec<Entity> = Vec::with_capacity(child_count);
        let mut prev_is_flexi = true;

        for i in 0..child_count {
            let Ok(flexi) = q_flexi_rows.get(children[i]) else {
                if let Ok(style) = q_style.get(children[i]) {
                    if style.position_type == PositionType::Relative {
                        prev_is_flexi = false;
                    }
                }
                continue;
            };

            commands
                .entity(flexi.top_handle)
                .style()
                .visibility(match prev_is_flexi {
                    true => Visibility::Hidden,
                    false => Visibility::Visible,
                });

            commands
                .entity(flexi.bottom_handle)
                .style()
                .visibility(match i == child_count - 1 {
                    true => Visibility::Hidden,
                    false => Visibility::Visible,
                });

            prev_is_flexi = true;
            flexi_children.push(children[i]);
        }

        for i in 0..flexi_children.len() {
            let flexi = q_flexi_rows.get(flexi_children[i]).unwrap();
            let top_handle = flexi.top_handle;
            let bottom_handle = flexi.bottom_handle;

            let mut top_handle = q_resize_handle.get_mut(top_handle).unwrap();
            top_handle.neighbour = if i > 0 {
                flexi_children[i - 1].into()
            } else {
                None
            };
            top_handle.flexi_row = flexi_children[i].into();

            let mut bottom_handle = q_resize_handle.get_mut(bottom_handle).unwrap();
            bottom_handle.flexi_row = flexi_children[i].into();
            bottom_handle.neighbour = if i < flexi_children.len() - 1 {
                flexi_children[i + 1].into()
            } else {
                None
            };
        }
    }
}

// TODO: Consider children min_height for constraints
fn update_flexi_row_on_resize(
    q_draggable: Query<(&Draggable, &ResizeHandle, &FlexiRowResizeHandle), Changed<Draggable>>,
    mut q_flexi_rows: Query<(&mut FlexiRow, Option<&Parent>)>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_node: Query<&Node>,
) {
    for (draggable, handle, handle_ref) in &q_draggable {
        if handle_ref.flexi_row.is_none() || handle_ref.neighbour.is_none() {
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

        let size_diff = handle.direction().to_size_diff(diff).y;
        if size_diff == 0. {
            continue;
        }

        let current_row_id = handle_ref.flexi_row.unwrap();
        let neighbour_row_id = handle_ref.neighbour.unwrap();

        let Ok((current_row, parent)) = q_flexi_rows.get(current_row_id) else {
            continue;
        };
        let Ok((neighbour_row, other_parent)) = q_flexi_rows.get(neighbour_row_id) else {
            continue;
        };

        if parent != other_parent {
            warn!(
                "Failed to resize flexi row: Rows have different parents: {:?} <-> {:?}",
                parent, other_parent
            );
            continue;
        }

        let total_height;
        if let Some(parent) = parent {
            let Ok(node) = q_node.get(parent.get()) else {
                warn!(
                    "Cannot calculate FlexiRow pixel height: Entity {:?} has parent without Node!",
                    current_row_id
                );
                continue;
            };

            total_height = node.size().y;
        } else {
            // TODO: Use node's render window
            let Ok(window) = q_window.get_single() else {
                warn!(
                    "Cannot calculate FlexiRow {:?} pixel height: Missing window!",
                    current_row_id
                );
                continue;
            };
            total_height = window.resolution.height();
        }

        if total_height == 0. {
            continue;
        }

        let current_min_height = current_row.min_height;
        let current_height = (current_row.height_percent / 100.) * total_height;
        let mut current_new_height = current_height;
        let neighbour_min_height = neighbour_row.min_height;
        let neighbour_height = (neighbour_row.height_percent / 100.) * total_height;
        let mut neighbour_new_height = neighbour_height;

        if size_diff < 0. {
            if current_height + size_diff >= current_min_height {
                current_new_height += size_diff;
                neighbour_new_height -= size_diff;
            } else {
                current_new_height = current_min_height;
                neighbour_new_height += current_height - current_min_height;
            }
        } else if size_diff > 0. {
            if neighbour_height - size_diff >= neighbour_min_height {
                neighbour_new_height -= size_diff;
                current_new_height += size_diff;
            } else {
                neighbour_new_height = neighbour_min_height;
                current_new_height += neighbour_height - neighbour_min_height;
            }
        }

        q_flexi_rows
            .get_mut(current_row_id)
            .unwrap()
            .0
            .height_percent = (current_new_height / total_height) * 100.;

        q_flexi_rows
            .get_mut(neighbour_row_id)
            .unwrap()
            .0
            .height_percent = (neighbour_new_height / total_height) * 100.;
    }
}

fn update_flexi_row_height(mut q_flexi_rows: Query<(&FlexiRow, &mut Style), Changed<FlexiRow>>) {
    for (config, mut style) in &mut q_flexi_rows {
        style.height = Val::Percent(config.height_percent);
    }
}

fn should_fit_flexi_rows(
    q_added_rows: Query<Entity, Added<FlexiRow>>,
    mut q_removed_rows: RemovedComponents<FlexiRow>,
    mut e_resize: EventReader<WindowResized>,
) -> bool {
    q_added_rows.iter().count() > 0
        || q_removed_rows.read().count() > 0
        || e_resize.read().count() > 0
}

fn fit_flexi_rows_on_window_resize(
    q_children: Query<&Children>,
    q_node: Query<&Node>,
    q_flexi_row_parents: Query<&Parent, With<FlexiRow>>,
    q_non_flexi: Query<(&Node, &Style), Without<FlexiRow>>,
    q_root_flexi_rows: Query<Entity, (With<FlexiRow>, Without<Parent>)>,
    q_non_flexi_root: Query<(&Node, &Style), (Without<FlexiRow>, Without<Parent>)>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    mut q_flexi_row: Query<(&mut FlexiRow, &Node)>,
) {
    let parents: Vec<Entity> = q_flexi_row_parents.iter().fold(
        Vec::with_capacity(q_flexi_row_parents.iter().count()),
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
            warn!("Flexi row parent {:?} doesn't have a Node!", parent);
            continue;
        };

        if node.size().y == 0. {
            warn!("Flexi row parent {:?} doesn't have a size!", parent);
            continue;
        }

        let mut non_flexi_height = 0.;
        for child in q_children.get(parent).unwrap().iter() {
            if let Ok((node, style)) = q_non_flexi.get(*child) {
                if style.position_type == PositionType::Relative {
                    non_flexi_height += node.size().y;
                }
            }
        }

        let mut sum_flexi_height = 0.;
        for child in q_children.get(parent).unwrap().iter() {
            if let Ok((_, node)) = q_flexi_row.get(*child) {
                sum_flexi_height += node.size().y;
            };
        }

        let total_height = node.size().y;
        for child in q_children.get(parent).unwrap().iter() {
            update_flexi_row_height_percent(
                *child,
                total_height,
                non_flexi_height,
                sum_flexi_height,
                &mut q_flexi_row,
            );
        }
    }

    if q_root_flexi_rows.iter().count() > 0 {
        // TODO: Use node's render window
        let Ok(window) = q_window.get_single() else {
            warn!("Cannot update FlexiColumn width: Missing window!",);
            return;
        };

        let non_flexi_height = q_non_flexi_root.iter().fold(0., |acc, (node, style)| {
            if style.position_type == PositionType::Relative {
                acc + node.size().y
            } else {
                acc
            }
        });

        let sum_flexi_height = q_root_flexi_rows.iter().fold(0., |acc, entity| {
            let Ok((_, node)) = q_flexi_row.get(entity) else {
                return acc;
            };
            acc + node.size().y
        });

        let total_height = window.resolution.height();
        for entity in &q_root_flexi_rows {
            update_flexi_row_height_percent(
                entity,
                total_height,
                non_flexi_height,
                sum_flexi_height,
                &mut q_flexi_row,
            );
        }
    }
}

fn update_flexi_row_height_percent(
    entity: Entity,
    total_height: f32,
    non_flexi_height: f32,
    sum_flexi_height: f32,
    q_flexi_row: &mut Query<(&mut FlexiRow, &Node)>,
) {
    let Ok((mut flexi_row, node)) = q_flexi_row.get_mut(entity) else {
        return;
    };

    let flexi_height = total_height - non_flexi_height;
    let multiplier = flexi_height / sum_flexi_height;

    flexi_row.height_percent =
        (node.size().y.max(flexi_row.min_height) / flexi_height) * 100. * multiplier;
}

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct FlexiRowResizeHandle {
    pub flexi_row: Option<Entity>,
    pub neighbour: Option<Entity>,
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct FlexiRow {
    height_percent: f32,
    min_height: f32,
    top_handle: Entity,
    bottom_handle: Entity,
}

impl Default for FlexiRow {
    fn default() -> Self {
        Self {
            height_percent: Default::default(),
            min_height: MIN_FLEXI_ROW_HEIGHT,
            top_handle: Entity::PLACEHOLDER,
            bottom_handle: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Debug, Default)]
pub struct FlexiRowConfig {
    pub height: f32,
    pub min_height: f32,
    pub background_color: Color,
}

impl FlexiRow {
    fn frame() -> impl Bundle {
        NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                border: UiRect::vertical(Val::Px(2.)),
                ..default()
            },
            border_color: Color::BLACK.into(),
            ..default()
        }
    }
}

impl Into<RowConfig> for FlexiRowConfig {
    fn into(self) -> RowConfig {
        RowConfig {
            height: Val::Percent(100.),
            background_color: self.background_color,
            ..default()
        }
    }
}

pub trait UiFlexiRowExt<'w, 's> {
    fn flexi_row<'a>(
        &'a mut self,
        config: FlexiRowConfig,
        spawn_children: impl FnOnce(&mut UiBuilder),
    ) -> EntityCommands<'w, 's, 'a>;
}

impl<'w, 's> UiFlexiRowExt<'w, 's> for UiBuilder<'w, 's, '_> {
    fn flexi_row<'a>(
        &'a mut self,
        config: FlexiRowConfig,
        spawn_children: impl FnOnce(&mut UiBuilder),
    ) -> EntityCommands<'w, 's, 'a> {
        let height = config.height.clamp(0., 100.);
        let min_height = config.min_height.max(MIN_FLEXI_ROW_HEIGHT);
        let mut top_handle = Entity::PLACEHOLDER;
        let mut bottom_handle = Entity::PLACEHOLDER;
        let row = self
            .container(FlexiRow::frame(), |container| {
                container.row(config.into(), spawn_children);
                container.container(
                    ResizeHandle::resize_handle_container(),
                    |resize_container| {
                        top_handle = resize_container
                            .spawn((
                                ResizeHandle::resize_handle(ResizeDirection::North),
                                FlexiRowResizeHandle { ..default() },
                            ))
                            .id();
                        bottom_handle = resize_container
                            .spawn((
                                ResizeHandle::resize_handle(ResizeDirection::South),
                                FlexiRowResizeHandle { ..default() },
                            ))
                            .id();
                    },
                );
            })
            .insert(FlexiRow {
                height_percent: height,
                min_height,
                top_handle,
                bottom_handle,
                ..default()
            })
            .id();

        self.commands().entity(row)
    }
}
