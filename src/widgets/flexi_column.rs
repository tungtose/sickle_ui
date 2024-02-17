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

use super::prelude::{ColumnConfig, UiColumnExt, UiContainerExt};

const MIN_FLEXI_COLUMN_WIDTH: f32 = 50.;

pub struct FlexiColumnPlugin;

impl Plugin for FlexiColumnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            update_flexi_column_resize_handles.run_if(should_update_resize_handles),
        )
        .add_systems(
            Update,
            (
                update_flexi_column_on_resize.after(DraggableUpdate),
                update_flexi_column_width,
            )
                .chain(),
        )
        .add_systems(
            PostUpdate,
            fit_flexi_columns_on_window_resize
                .run_if(should_fit_flexi_columns)
                .after(UiSystem::Layout),
        );
    }
}

fn should_update_resize_handles(
    q_added_columns: Query<Entity, Added<FlexiColumn>>,
    mut q_removed_columns: RemovedComponents<FlexiColumn>,
) -> bool {
    q_added_columns.iter().count() > 0 || q_removed_columns.read().count() > 0
}

fn update_flexi_column_resize_handles(
    q_flexi_column_parents: Query<&Parent, With<FlexiColumn>>,
    q_children: Query<&Children>,
    q_root_flexi_columns: Query<Entity, (With<FlexiColumn>, Without<Parent>)>,
    q_non_flexi_root: Query<Entity, (With<Node>, Without<Parent>)>,
    q_flexi_columns: Query<&FlexiColumn>,
    q_style: Query<&Style>,
    mut q_resize_handle: Query<&mut FlexiColumnResizeHandle>,
    mut commands: Commands,
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
        let children: Vec<Entity> = q_children.get(parent).unwrap().iter().map(|e| *e).collect();
        update_flexi_column_handles_on_collection(
            children,
            &q_flexi_columns,
            &q_style,
            &mut q_resize_handle,
            &mut commands,
        );
    }

    if q_root_flexi_columns.iter().count() > 0 {
        let children: Vec<Entity> = q_non_flexi_root.iter().map(|e| e).collect();
        update_flexi_column_handles_on_collection(
            children,
            &q_flexi_columns,
            &q_style,
            &mut q_resize_handle,
            &mut commands,
        );
    }
}

fn update_flexi_column_handles_on_collection(
    children: Vec<Entity>,
    q_flexi_columns: &Query<&FlexiColumn>,
    q_style: &Query<&Style>,
    q_resize_handle: &mut Query<&mut FlexiColumnResizeHandle>,
    commands: &mut Commands,
) {
    let child_count = children.len();

    if child_count == 1 {
        let Ok(flexi) = q_flexi_columns.get(children[0]) else {
            return;
        };

        commands
            .entity(flexi.left_handle)
            .style()
            .visibility(Visibility::Hidden);
        commands
            .entity(flexi.right_handle)
            .style()
            .visibility(Visibility::Hidden);
    } else {
        let mut flexi_children: Vec<Entity> = Vec::with_capacity(child_count);
        let mut prev_is_flexi = true;

        for i in 0..child_count {
            let Ok(flexi) = q_flexi_columns.get(children[i]) else {
                if let Ok(style) = q_style.get(children[i]) {
                    if style.position_type == PositionType::Relative {
                        prev_is_flexi = false;
                    }
                }
                continue;
            };

            commands
                .entity(flexi.left_handle)
                .style()
                .visibility(match prev_is_flexi {
                    true => Visibility::Hidden,
                    false => Visibility::Visible,
                });

            commands
                .entity(flexi.right_handle)
                .style()
                .visibility(match i == child_count - 1 {
                    true => Visibility::Hidden,
                    false => Visibility::Visible,
                });

            prev_is_flexi = true;
            flexi_children.push(children[i]);
        }

        for i in 0..flexi_children.len() {
            let flexi = q_flexi_columns.get(flexi_children[i]).unwrap();
            let left_handle = flexi.left_handle;
            let right_handle = flexi.right_handle;

            let mut left_handle = q_resize_handle.get_mut(left_handle).unwrap();
            left_handle.neighbour = if i > 0 {
                flexi_children[i - 1].into()
            } else {
                None
            };
            left_handle.flexi_column = flexi_children[i].into();

            let mut right_handle = q_resize_handle.get_mut(right_handle).unwrap();
            right_handle.flexi_column = flexi_children[i].into();
            right_handle.neighbour = if i < flexi_children.len() - 1 {
                flexi_children[i + 1].into()
            } else {
                None
            };
        }
    }
}

// TODO: Consider children min_width for constraints
fn update_flexi_column_on_resize(
    q_draggable: Query<(&Draggable, &ResizeHandle, &FlexiColumnResizeHandle), Changed<Draggable>>,
    mut q_flexi_columns: Query<(&mut FlexiColumn, Option<&Parent>)>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_node: Query<&Node>,
) {
    for (draggable, handle, handle_ref) in &q_draggable {
        if handle_ref.flexi_column.is_none() || handle_ref.neighbour.is_none() {
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

        let current_column_id = handle_ref.flexi_column.unwrap();
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

        let total_width;
        if let Some(parent) = parent {
            let Ok(node) = q_node.get(parent.get()) else {
                warn!("Cannot calculate FlexiColumn pixel width: Entity {:?} has parent without Node!", current_column_id);
                continue;
            };

            total_width = node.size().x;
        } else {
            // TODO: Use node's render window
            let Ok(window) = q_window.get_single() else {
                warn!(
                    "Cannot calculate FlexiColumn {:?} pixel width: Missing window!",
                    current_column_id
                );
                continue;
            };
            total_width = window.resolution.width();
        }

        if total_width == 0. {
            continue;
        }

        let current_min_width = current_column.min_width;
        let current_width = (current_column.width_percent / 100.) * total_width;
        let mut current_new_width = current_width;
        let neighbour_min_width = neighbour_column.min_width;
        let neighbour_width = (neighbour_column.width_percent / 100.) * total_width;
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
            .width_percent = (current_new_width / total_width) * 100.;

        q_flexi_columns
            .get_mut(neighbour_column_id)
            .unwrap()
            .0
            .width_percent = (neighbour_new_width / total_width) * 100.;
    }
}

fn update_flexi_column_width(
    mut q_flexi_columns: Query<(&FlexiColumn, &mut Style), Changed<FlexiColumn>>,
) {
    for (config, mut style) in &mut q_flexi_columns {
        style.width = Val::Percent(config.width_percent);
    }
}

fn should_fit_flexi_columns(
    q_added_columns: Query<Entity, Added<FlexiColumn>>,
    mut q_removed_columns: RemovedComponents<FlexiColumn>,
    mut e_resize: EventReader<WindowResized>,
) -> bool {
    q_added_columns.iter().count() > 0
        || q_removed_columns.read().count() > 0
        || e_resize.read().count() > 0
}

fn fit_flexi_columns_on_window_resize(
    q_children: Query<&Children>,
    q_node: Query<&Node>,
    q_flexi_column_parents: Query<&Parent, With<FlexiColumn>>,
    q_non_flexi: Query<(&Node, &Style), Without<FlexiColumn>>,
    q_root_flexi_columns: Query<Entity, (With<FlexiColumn>, Without<Parent>)>,
    q_non_flexi_root: Query<(&Node, &Style), (Without<FlexiColumn>, Without<Parent>)>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    mut q_flexi_column: Query<(&mut FlexiColumn, &Node)>,
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
            update_flexi_column_width_percent(
                *child,
                total_width,
                non_flexi_width,
                sum_flexi_width,
                &mut q_flexi_column,
            );
        }
    }

    if q_root_flexi_columns.iter().count() > 0 {
        // TODO: Use node's render window
        let Ok(window) = q_window.get_single() else {
            warn!("Cannot update FlexiColumn width: Missing window!",);
            return;
        };

        let _non_flexi_width = q_non_flexi_root.iter().fold(0., |acc, (node, style)| {
            if style.position_type == PositionType::Relative {
                acc + node.size().x
            } else {
                acc
            }
        });

        let sum_flexi_width = q_root_flexi_columns.iter().fold(0., |acc, entity| {
            let Ok((_, node)) = q_flexi_column.get(entity) else {
                return acc;
            };
            acc + node.size().x
        });

        let total_width = window.resolution.width();
        for entity in &q_root_flexi_columns {
            update_flexi_column_width_percent(
                entity,
                total_width,
                0.,
                sum_flexi_width,
                &mut q_flexi_column,
            );
        }
    }
}

fn update_flexi_column_width_percent(
    entity: Entity,
    total_width: f32,
    non_flexi_width: f32,
    sum_flexi_width: f32,
    q_flexi_column: &mut Query<(&mut FlexiColumn, &Node)>,
) {
    let Ok((mut flexi_column, node)) = q_flexi_column.get_mut(entity) else {
        return;
    };

    let flexi_width = total_width - non_flexi_width;
    let multiplier = flexi_width / sum_flexi_width;
    info!(
        "total: {:?}, non flexi: {:?}, flexi width: {:?}, mul: {:?}",
        total_width, non_flexi_width, flexi_width, multiplier
    );
    flexi_column.width_percent =
        (node.size().x.max(flexi_column.min_width) / flexi_width) * 100. * multiplier;
}

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct FlexiColumnResizeHandle {
    pub flexi_column: Option<Entity>,
    pub neighbour: Option<Entity>,
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct FlexiColumn {
    width_percent: f32,
    min_width: f32,
    left_handle: Entity,
    right_handle: Entity,
}

impl Default for FlexiColumn {
    fn default() -> Self {
        Self {
            width_percent: Default::default(),
            min_width: MIN_FLEXI_COLUMN_WIDTH,
            left_handle: Entity::PLACEHOLDER,
            right_handle: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Debug, Default)]
pub struct FlexiColumnConfig {
    pub width: f32,
    pub min_width: f32,
    pub background_color: Color,
}

impl FlexiColumn {
    fn frame() -> impl Bundle {
        NodeBundle {
            style: Style {
                height: Val::Percent(100.),
                flex_direction: FlexDirection::Column,
                border: UiRect::horizontal(Val::Px(2.)),
                ..default()
            },
            border_color: Color::BLACK.into(),
            ..default()
        }
    }
}

impl Into<ColumnConfig> for FlexiColumnConfig {
    fn into(self) -> ColumnConfig {
        ColumnConfig {
            width: Val::Percent(100.),
            background_color: self.background_color,
            ..default()
        }
    }
}

pub trait UiFlexiColumnExt<'w, 's> {
    fn flexi_column<'a>(
        &'a mut self,
        config: FlexiColumnConfig,
        spawn_children: impl FnOnce(&mut UiBuilder),
    ) -> EntityCommands<'w, 's, 'a>;
}

impl<'w, 's> UiFlexiColumnExt<'w, 's> for UiBuilder<'w, 's, '_> {
    fn flexi_column<'a>(
        &'a mut self,
        config: FlexiColumnConfig,
        spawn_children: impl FnOnce(&mut UiBuilder),
    ) -> EntityCommands<'w, 's, 'a> {
        let width = config.width.clamp(0., 100.);
        let min_width = config.min_width.max(MIN_FLEXI_COLUMN_WIDTH);
        let mut left_handle = Entity::PLACEHOLDER;
        let mut right_handle = Entity::PLACEHOLDER;

        if self.id().is_none() {
            warn!("Flexi column as root node is not supported! An additional static element has been injected.");
        }

        let mut root = match self.id().is_none() {
            true => self.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    ..default()
                },
                ..default()
            }),
            false => self.entity_commands().unwrap(),
        };

        let column = root
            .ui_builder()
            .container(FlexiColumn::frame(), |container| {
                container.column(config.into(), spawn_children);
                container.container(
                    ResizeHandle::resize_handle_container(),
                    |resize_container| {
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
                                left_handle = middle_row
                                    .spawn((
                                        ResizeHandle::resize_handle(ResizeDirection::West),
                                        FlexiColumnResizeHandle { ..default() },
                                    ))
                                    .id();
                                right_handle = middle_row
                                    .spawn((
                                        ResizeHandle::resize_handle(ResizeDirection::East),
                                        FlexiColumnResizeHandle { ..default() },
                                    ))
                                    .id();
                            },
                        );
                    },
                );
            })
            .insert(FlexiColumn {
                width_percent: width,
                min_width,
                left_handle,
                right_handle,
                ..default()
            })
            .id();

        self.commands().entity(column)
    }
}
