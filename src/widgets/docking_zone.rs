use bevy::{
    ecs::system::{Command, CommandQueue},
    prelude::*,
};
use floating_panel::FloatingPanelConfig;

use crate::{
    drag_interaction::{DragState, Draggable},
    drop_interaction::{DropPhase, DropZone, DroppableUpdate},
    ui_builder::{UiBuilder, UiBuilderExt},
    ui_style::{
        SetBackgroundColorExt, SetNodeHeightExt, SetNodeLeftExt, SetNodeShowHideExt, SetNodeTopExt,
        SetNodeWidthExt, UiStyleExt,
    },
};

use super::{
    floating_panel::{self, FloatingPanel, FloatingPanelTitle},
    panel::Panel,
    prelude::{SizedZoneConfig, UiPanelExt, UiSizedZoneExt, UiTabContainerExt},
    sized_zone::{SizedZone, SizedZoneResizeHandleContainer},
    tab_container::{TabBar, TabContainer},
};

pub struct DockingZonePlugin;

impl Plugin for DockingZonePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Update, DockingZoneUpdate.after(DroppableUpdate))
            .add_systems(
                Update,
                (
                    update_docking_zone_resize_handles.run_if(should_update_resize_handles),
                    handle_docking_zone_drop_zone_change,
                )
                    .in_set(DockingZoneUpdate),
            );
    }
}

#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct DockingZoneUpdate;

fn should_update_resize_handles(
    q_accepted_types: Query<&Draggable, (With<FloatingPanelTitle>, Changed<Draggable>)>,
) -> bool {
    q_accepted_types
        .iter()
        .any(|draggable| draggable.state != DragState::Inactive)
}

fn update_docking_zone_resize_handles(
    q_accepted_types: Query<&Draggable, (With<FloatingPanelTitle>, Changed<Draggable>)>,
    q_handle_containers: Query<Entity, With<SizedZoneResizeHandleContainer>>,
    mut commands: Commands,
) {
    let dragging = q_accepted_types.iter().any(|draggable| {
        draggable.state == DragState::DragStart || draggable.state == DragState::Dragging
    });

    for container in &q_handle_containers {
        commands.style(container).render(!dragging);
    }
}

fn handle_docking_zone_drop_zone_change(
    q_docking_zones: Query<
        (Entity, &DockingZone, &DropZone, &Node, &GlobalTransform),
        Changed<DropZone>,
    >,
    q_accepted_query: Query<&FloatingPanelTitle>,
    q_tab_container: Query<&TabContainer>,
    q_tab_bar: Query<(&Node, &Interaction), With<TabBar>>,
    mut commands: Commands,
) {
    for (entity, docking_zone, drop_zone, node, transform) in &q_docking_zones {
        let Ok(tab_container) = q_tab_container.get(docking_zone.tab_container) else {
            warn!("Docking zone {:?} missing its tab container!", entity);
            continue;
        };

        let Ok((tab_bar_node, bar_interaction)) = q_tab_bar.get(tab_container.bar_id()) else {
            warn!(
                "Tab container {:?} missing its tab bar!",
                docking_zone.tab_container
            );
            continue;
        };

        let center = transform.translation().truncate();
        let tab_bar_height = tab_bar_node.size().y;

        if *bar_interaction == Interaction::Hovered
            || drop_zone.drop_phase() == DropPhase::Inactive
            || drop_zone.drop_phase() == DropPhase::DropCanceled
            || drop_zone.drop_phase() == DropPhase::DroppableLeft
            || drop_zone.incoming_droppable().is_none()
            || q_accepted_query
                .get(drop_zone.incoming_droppable().unwrap())
                .is_err()
        {
            commands
                .style(docking_zone.zone_highlight)
                .background_color(Color::NONE);

            continue;
        }

        // How else would the droppable be over the zone?
        let position = drop_zone.position().unwrap();
        let drop_area = calculate_drop_area(position, center, node.size());

        if drop_zone.drop_phase() == DropPhase::DroppableEntered
            || drop_zone.drop_phase() == DropPhase::DroppableHover
        {
            let full_size = Val::Percent(100.);
            let half_size = Val::Percent(50.);
            let auto_size = Val::Auto;

            let (width, height, top, left) = match drop_area {
                DropArea::Center => (
                    full_size,
                    Val::Px(node.size().y - tab_bar_height),
                    Val::Px(tab_bar_height),
                    auto_size,
                ),
                DropArea::North => (full_size, half_size, auto_size, auto_size),
                DropArea::East => (half_size, full_size, auto_size, half_size),
                DropArea::South => (full_size, half_size, half_size, auto_size),
                DropArea::West => (half_size, full_size, auto_size, auto_size),
                _ => (full_size, full_size, auto_size, auto_size),
            };

            commands
                .style(docking_zone.zone_highlight)
                .width(width)
                .height(height)
                .left(left)
                .top(top)
                .background_color(Color::rgba(0.7, 0.8, 0.9, 0.2));
        } else if drop_zone.drop_phase() == DropPhase::Dropped {
            // Validated above
            let droppable_title = q_accepted_query
                .get(drop_zone.incoming_droppable().unwrap())
                .unwrap();

            if drop_area == DropArea::Center {
                commands.add(MoveFloatingPanelToDockingZone {
                    floating_panel_id: droppable_title.panel(),
                    target_container_id: docking_zone.tab_container,
                });
            } else {
                let split_direction = match drop_area {
                    DropArea::North => DockingZoneSplitDirection::VerticallyBefore,
                    DropArea::East => DockingZoneSplitDirection::HorizontallyAfter,
                    DropArea::South => DockingZoneSplitDirection::VerticallyAfter,
                    DropArea::West => DockingZoneSplitDirection::HorizontallyBefore,
                    _ => DockingZoneSplitDirection::VerticallyAfter,
                };

                commands.add(DockingZoneSplit {
                    direction: split_direction,
                    docking_zone: entity,
                    panel_to_dock: droppable_title.panel().into(),
                });
            }

            commands
                .style(docking_zone.zone_highlight)
                .background_color(Color::NONE);
        }
    }
}

fn calculate_drop_area(position: Vec2, center: Vec2, size: Vec2) -> DropArea {
    let sixth_width = size.x / 6.;
    let sixth_height = size.y / 6.;

    if position.x < center.x - sixth_width {
        DropArea::West
    } else if position.x > center.x + sixth_width {
        DropArea::East
    } else if position.y < center.y - sixth_height {
        DropArea::North
    } else if position.y > center.y + sixth_height {
        DropArea::South
    } else {
        DropArea::Center
    }
}

#[derive(PartialEq, Eq)]
enum DockingZoneSplitDirection {
    VerticallyBefore,
    VerticallyAfter,
    HorizontallyBefore,
    HorizontallyAfter,
}

struct DockingZoneSplit {
    docking_zone: Entity,
    direction: DockingZoneSplitDirection,
    panel_to_dock: Option<Entity>,
}

impl Command for DockingZoneSplit {
    fn apply(self, world: &mut World) {
        let Ok((docking_zone, parent, sized_zone)) = world
            .query::<(&DockingZone, &Parent, &SizedZone)>()
            .get(world, self.docking_zone)
        else {
            error!(
                "Tried to split entity {:?} when it isn't a valid DockingZone!",
                self.docking_zone
            );
            return;
        };

        let tab_container_id = docking_zone.tab_container;
        let mut parent_id = parent.get();
        let current_direction = sized_zone.direction();
        let current_size = sized_zone.size();
        let current_min_size = sized_zone.min_size();
        let Ok(_) = world.query::<&TabContainer>().get(world, tab_container_id) else {
            error!(
                "Tab container {:?} missing from docking zone {:?}",
                tab_container_id, self.docking_zone
            );
            return;
        };

        // This must exists, since the Parent exists
        let current_index = world
            .query::<&Children>()
            .get(world, parent_id)
            .unwrap()
            .iter()
            .position(|child| *child == self.docking_zone)
            .unwrap();

        let (inject_container, sibling_before) = match current_direction {
            FlexDirection::Row => match self.direction {
                DockingZoneSplitDirection::VerticallyBefore => (false, true),
                DockingZoneSplitDirection::VerticallyAfter => (false, false),
                DockingZoneSplitDirection::HorizontallyBefore => (true, true),
                DockingZoneSplitDirection::HorizontallyAfter => (true, false),
            },
            FlexDirection::Column => match self.direction {
                DockingZoneSplitDirection::VerticallyBefore => (true, true),
                DockingZoneSplitDirection::VerticallyAfter => (true, false),
                DockingZoneSplitDirection::HorizontallyBefore => (false, true),
                DockingZoneSplitDirection::HorizontallyAfter => (false, false),
            },
            FlexDirection::RowReverse => match self.direction {
                DockingZoneSplitDirection::VerticallyBefore => (false, false),
                DockingZoneSplitDirection::VerticallyAfter => (false, true),
                DockingZoneSplitDirection::HorizontallyBefore => (true, false),
                DockingZoneSplitDirection::HorizontallyAfter => (true, true),
            },
            FlexDirection::ColumnReverse => match self.direction {
                DockingZoneSplitDirection::VerticallyBefore => (true, false),
                DockingZoneSplitDirection::VerticallyAfter => (true, true),
                DockingZoneSplitDirection::HorizontallyBefore => (false, false),
                DockingZoneSplitDirection::HorizontallyAfter => (false, true),
            },
        };

        let mut sized_zone = world
            .query::<&mut SizedZone>()
            .get_mut(world, self.docking_zone)
            .unwrap();

        let new_container_size = if inject_container {
            50.
        } else {
            current_size / 2.
        };
        sized_zone.set_size(new_container_size);

        let mut queue = CommandQueue::default();
        let mut commands = Commands::new(&mut queue, world);
        let mut new_container = Entity::PLACEHOLDER;

        if inject_container {
            let new_parent_id = commands
                .ui_builder(parent_id.into())
                .sized_zone(
                    SizedZoneConfig {
                        size: current_size,
                        min_size: current_min_size,
                        ..default()
                    },
                    |_| {},
                )
                .id();
            commands
                .entity(parent_id)
                .insert_children(current_index, &[new_parent_id]);

            parent_id = new_parent_id;
        }

        let new_docking_zone_id = commands
            .ui_builder(parent_id.into())
            .docking_zone(
                SizedZoneConfig {
                    size: new_container_size,
                    min_size: current_min_size,
                    ..default()
                },
                |tab_container| {
                    new_container = tab_container.id();
                },
            )
            .id();

        if inject_container {
            if sibling_before {
                commands.entity(parent_id).add_child(self.docking_zone);
            } else {
                commands
                    .entity(parent_id)
                    .insert_children(0, &[self.docking_zone]);
            }
        } else {
            if sibling_before {
                commands
                    .entity(parent_id)
                    .insert_children(current_index, &[new_docking_zone_id]);
            } else {
                commands
                    .entity(parent_id)
                    .insert_children(current_index + 1, &[new_docking_zone_id]);
            }
        }

        if let Some(floating_panel_id) = self.panel_to_dock {
            commands.add(MoveFloatingPanelToDockingZone {
                target_container_id: new_container,
                floating_panel_id,
            });
        }

        queue.apply(world);
    }
}

struct MoveFloatingPanelToDockingZone {
    floating_panel_id: Entity,
    target_container_id: Entity,
}

impl Command for MoveFloatingPanelToDockingZone {
    fn apply(self, world: &mut World) {
        let Ok((floating_panel, panel_config)) = world
            .query::<(&FloatingPanel, &FloatingPanelConfig)>()
            .get(world, self.floating_panel_id)
        else {
            warn!(
                "Cannot dock {:?} in new DockingZone: Not a FloatingPanel!",
                self.floating_panel_id
            );
            return;
        };

        let title = panel_config.title().unwrap_or("Untitled".into());
        let content_container_id = floating_panel.content_container_id();

        let Ok(mut tab_container) = world
            .query::<&mut TabContainer>()
            .get_mut(world, self.target_container_id)
        else {
            error!(
                "Target container {:?} isn't a TabContainer!",
                self.target_container_id
            );
            return;
        };

        let tab_count = tab_container.tab_count();
        tab_container.set_active(tab_count);

        let Ok(children) = world.query::<&Children>().get(world, content_container_id) else {
            warn!(
                "Cannot dock {:?} in new DockingZone: Nothing to dock!",
                self.floating_panel_id
            );
            return;
        };

        let children_ids: Vec<Entity> = children.iter().map(|child| *child).collect();
        let mut panel_to_move = Entity::PLACEHOLDER;
        if let Some(panel_id) = children_ids
            .iter()
            .find(|child| world.query::<&Panel>().get(world, **child).is_ok())
        {
            panel_to_move = panel_id.to_owned();
        }

        let mut queue = CommandQueue::default();
        let mut commands = Commands::new(&mut queue, world);

        if panel_to_move == Entity::PLACEHOLDER {
            commands
                .ui_builder(self.target_container_id.into())
                .panel(title, |_| {})
                .entity_commands()
                .insert_children(0, &children_ids);
        } else {
            commands
                .entity(self.target_container_id)
                .add_child(panel_to_move);
        }

        commands.entity(self.floating_panel_id).despawn_recursive();
        queue.apply(world);
    }
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
enum DropArea {
    #[default]
    None,
    Center,
    North,
    East,
    South,
    West,
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct DockingZone {
    tab_container: Entity,
    zone_highlight: Entity,
}

impl Default for DockingZone {
    fn default() -> Self {
        Self {
            tab_container: Entity::PLACEHOLDER,
            zone_highlight: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct DockingZoneHighlight {
    zone: Entity,
}

impl Default for DockingZoneHighlight {
    fn default() -> Self {
        Self {
            zone: Entity::PLACEHOLDER,
        }
    }
}

impl DockingZone {
    fn zone_highlight() -> impl Bundle {
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                ..default()
            },
            background_color: Color::NONE.into(),
            z_index: ZIndex::Local(100),
            ..default()
        }
    }
}

pub trait UiDockingZoneExt<'w, 's> {
    fn docking_zone<'a>(
        &'a mut self,
        config: SizedZoneConfig,
        spawn_children: impl FnOnce(&mut UiBuilder),
    ) -> UiBuilder<'w, 's, 'a>;
}

impl<'w, 's> UiDockingZoneExt<'w, 's> for UiBuilder<'w, 's, '_> {
    fn docking_zone<'a>(
        &'a mut self,
        config: SizedZoneConfig,
        spawn_children: impl FnOnce(&mut UiBuilder),
    ) -> UiBuilder<'w, 's, 'a> {
        let mut tab_container = Entity::PLACEHOLDER;
        let mut zone_highlight = Entity::PLACEHOLDER;

        let mut docking_zone = self.sized_zone(config, |zone| {
            let zone_id = zone.id();
            tab_container = zone.tab_container(spawn_children).id();
            zone_highlight = zone
                .spawn((
                    DockingZone::zone_highlight(),
                    DockingZoneHighlight { zone: zone_id },
                ))
                .id();
        });

        docking_zone.insert((
            DockingZone {
                tab_container,
                zone_highlight,
            },
            Interaction::default(),
            DropZone::default(),
        ));

        docking_zone
    }
}
