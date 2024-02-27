use bevy::prelude::*;

use crate::{
    drag_interaction::{DragState, Draggable, DraggableUpdate},
    drop_interaction::{DropPhase, DropZone, DroppableUpdate},
    ui_builder::UiBuilder,
    ui_style::{
        SetBackgroundColorExt, SetNodeHeightExt, SetNodeLeftExt, SetNodeShowHideExt, SetNodeTopExt,
        SetNodeWidthExt, UiStyleExt,
    },
};

use super::{
    floating_panel::FloatingPanelTitle,
    prelude::{SizedZoneConfig, UiSizedZoneExt, UiTabContainerExt},
    sized_zone::SizedZoneResizeHandleContainer,
};

pub struct DockingZonePlugin;

impl Plugin for DockingZonePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_docking_zone_resize_handles
                    .run_if(should_update_resize_handles)
                    .after(DraggableUpdate),
                update_docking_zone_highlight.after(DroppableUpdate),
            ),
        );
    }
}

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

fn update_docking_zone_highlight(
    q_docking_zones: Query<(&DockingZone, &DropZone, &Node, &GlobalTransform), Changed<DropZone>>,
    q_accepted_types: Query<Entity, With<FloatingPanelTitle>>,
    mut commands: Commands,
) {
    for (docking_zone, drop_zone, node, transform) in &q_docking_zones {
        if drop_zone.drop_phase() == DropPhase::DroppableLeft
            || drop_zone.drop_phase() == DropPhase::DropCanceled
            || drop_zone.drop_phase() == DropPhase::Inactive
            || drop_zone.incoming_droppable().is_none()
            || q_accepted_types
                .get(drop_zone.incoming_droppable().unwrap())
                .is_err()
        {
            commands
                .style(docking_zone.zone_highlight)
                .width(Val::Percent(100.))
                .height(Val::Percent(100.))
                .left(Val::Auto)
                .top(Val::Auto)
                .background_color(Color::NONE);

            continue;
        }

        if drop_zone.drop_phase() == DropPhase::DroppableEntered {
            commands
                .style(docking_zone.zone_highlight)
                .background_color(Color::rgba(0.7, 0.8, 0.9, 0.2));
        }

        if drop_zone.drop_phase() == DropPhase::DroppableEntered
            || drop_zone.drop_phase() == DropPhase::DroppableHover
        {
            let center = transform.translation().truncate();
            // How else would the droppable be over the zone?
            let position = drop_zone.position().unwrap();
            let sixth_width = node.size().x / 6.;
            let sixth_height = node.size().y / 6.;

            let (width, height, top, left) = if position.x < center.x - sixth_width {
                (Val::Percent(50.), Val::Percent(100.), Val::Auto, Val::Auto)
            } else if position.x > center.x + sixth_width {
                (
                    Val::Percent(50.),
                    Val::Percent(100.),
                    Val::Auto,
                    Val::Percent(50.),
                )
            } else if position.y < center.y - sixth_height {
                (Val::Percent(100.), Val::Percent(50.), Val::Auto, Val::Auto)
            } else if position.y > center.y + sixth_height {
                (
                    Val::Percent(100.),
                    Val::Percent(50.),
                    Val::Percent(50.),
                    Val::Auto,
                )
            } else {
                (Val::Percent(100.), Val::Percent(100.), Val::Auto, Val::Auto)
            };

            commands
                .style(docking_zone.zone_highlight)
                .width(width)
                .height(height)
                .left(left)
                .top(top);
        }
    }
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
