use bevy::prelude::*;

use crate::drag_interaction::{DragState, Draggable, DraggableUpdate};

pub struct DropInteractionPlugin;

impl Plugin for DropInteractionPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Update, DroppableUpdate.after(DraggableUpdate))
            .add_systems(Update, update_drop_zones.chain().in_set(DroppableUpdate));
    }
}

#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct DroppableUpdate;

// TODO: Use node stack index to only interact with the topmost zone
fn update_drop_zones(
    q_droppables: Query<(Entity, &Draggable), (With<Droppable>, Changed<Draggable>)>,
    mut q_drop_zones: Query<(Entity, Ref<Interaction>, &mut DropZone, &Node)>,
) {
    if !q_droppables
        .iter()
        .any(|(_, Draggable { state, .. })| *state != DragState::MaybeDragged)
    {
        return;
    }

    let mut hovered_to_stack_index: Vec<(Entity, u32)> = q_drop_zones
        .iter()
        .filter(|(_, interaction, _, _)| **interaction == Interaction::Hovered)
        .map(|(entity, _, _, node)| (entity, node.stack_index()))
        .collect();

    hovered_to_stack_index.sort_by_key(|(_, i)| *i);
    let top_hovered = hovered_to_stack_index
        .last()
        .unwrap_or(&(Entity::PLACEHOLDER, 0u32))
        .0;

    for (_, interaction, mut drop_zone, _) in &mut q_drop_zones {
        if drop_zone.drop_phase == DropPhase::Dropped
            || drop_zone.drop_phase == DropPhase::DropCanceled
        {
            drop_zone.drop_phase = DropPhase::Inactive;
            drop_zone.incoming_droppable = None;
            drop_zone.position = None;
        } else if *interaction == Interaction::None {
            if drop_zone.drop_phase == DropPhase::DroppableHover {
                drop_zone.drop_phase = DropPhase::DroppableLeft;
            } else if drop_zone.drop_phase == DropPhase::DroppableLeft {
                drop_zone.drop_phase = DropPhase::Inactive;
                drop_zone.incoming_droppable = None;
                drop_zone.position = None;
            }
        } else if *interaction == Interaction::Hovered {
            if drop_zone.drop_phase == DropPhase::Inactive {
                drop_zone.drop_phase = DropPhase::DroppableEntered;
            } else if drop_zone.drop_phase == DropPhase::DroppableEntered {
                drop_zone.drop_phase = DropPhase::DroppableHover;
            }
        }
    }

    for (droppable_entity, draggable) in &q_droppables {
        if draggable.state == DragState::Inactive || draggable.state == DragState::MaybeDragged {
            continue;
        }

        if draggable.state == DragState::DragStart || draggable.state == DragState::Dragging {
            for (_, interaction, mut drop_zone, _) in &mut q_drop_zones {
                if interaction.is_changed() {
                    if *interaction == Interaction::Hovered {
                        drop_zone.drop_phase = DropPhase::DroppableEntered;
                        drop_zone.incoming_droppable = droppable_entity.into();
                        drop_zone.position = draggable.position;
                    } else if *interaction == Interaction::None {
                        drop_zone.drop_phase = DropPhase::DroppableLeft;
                        drop_zone.incoming_droppable = None;
                        drop_zone.position = None;
                    }
                } else if *interaction == Interaction::Hovered {
                    if drop_zone.drop_phase == DropPhase::Inactive {
                        drop_zone.drop_phase = DropPhase::DroppableEntered;
                        drop_zone.incoming_droppable = droppable_entity.into();
                    }
                    drop_zone.position = draggable.position;
                }
            }
        } else if draggable.state == DragState::DragEnd {
            for (entity, interaction, mut drop_zone, _) in &mut q_drop_zones {
                if *interaction == Interaction::Hovered {
                    if entity == top_hovered {
                        drop_zone.drop_phase = DropPhase::Dropped;
                        drop_zone.incoming_droppable = droppable_entity.into();
                        drop_zone.position = draggable.position;
                    } else {
                        drop_zone.drop_phase = DropPhase::DroppableLeft;
                        drop_zone.incoming_droppable = None;
                        drop_zone.position = None;
                    }
                }
            }
        } else {
            for (_, interaction, mut drop_zone, _) in &mut q_drop_zones {
                if *interaction == Interaction::Hovered {
                    drop_zone.drop_phase = DropPhase::DropCanceled;
                    drop_zone.incoming_droppable = None;
                    drop_zone.position = None;
                }
            }
        }
    }
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Reflect)]
#[reflect]
pub enum DropPhase {
    #[default]
    Inactive,
    DroppableEntered,
    DroppableHover,
    DroppableLeft,
    Dropped,
    DropCanceled,
}

#[derive(Component, Debug, Default, Reflect)]
pub struct Droppable;

#[derive(Component, Debug, Default, Reflect)]
pub struct DropZone {
    drop_phase: DropPhase,
    incoming_droppable: Option<Entity>,
    position: Option<Vec2>,
}

impl DropZone {
    pub fn drop_phase(&self) -> DropPhase {
        self.drop_phase
    }

    pub fn incoming_droppable(&self) -> Option<Entity> {
        self.incoming_droppable
    }

    pub fn position(&self) -> Option<Vec2> {
        self.position
    }
}