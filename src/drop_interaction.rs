use bevy::prelude::*;
use bevy_reflect::reflect_trait;

use crate::drag_interaction::{DragState, Draggable, DraggableUpdate};

pub struct DropInteractionPlugin;

impl Plugin for DropInteractionPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Update, DroppableUpdate.after(DraggableUpdate))
            .add_systems(Update, update_droppables.in_set(DroppableUpdate));
    }
}

#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct DroppableUpdate;

fn update_droppables(world: &mut World) {
    let mut q_droppables =
        world.query_filtered::<(Entity, &Draggable), (With<Droppable>, Changed<Draggable>)>();

    let droppable_count = q_droppables.iter(world).count();
    if droppable_count == 0 {
        return;
    }

    let changed_zones: Vec<(Entity, Interaction)> = world
        .query_filtered::<(Entity, &Interaction), (With<DropZone>, Changed<Interaction>)>()
        .iter(world)
        .map(|(entity, interaction)| (entity, *interaction))
        .collect();

    let hovered_zones: Vec<Entity> = world
        .query::<(Entity, Ref<Interaction>)>()
        .iter(world)
        .filter(|(_, interaction)| {
            **interaction == Interaction::Hovered && !interaction.is_changed()
        })
        .map(|(entity, _)| entity)
        .collect();

    if changed_zones.len() == 0 && hovered_zones.len() == 0 {
        return;
    }

    let active_droppables: Vec<(Entity, DragState, Vec2)> = q_droppables
        .iter(world)
        .filter(|(_, draggable)| {
            draggable.state != DragState::MaybeDragged && draggable.position.is_some()
        })
        .map(|(entity, draggable)| (entity, draggable.state.clone(), draggable.position.unwrap()))
        .collect();

    if active_droppables.len() == 0 {
        return;
    }

    
}

#[derive(Component, Debug, Default, Reflect)]
pub struct Droppable;

#[derive(Component, Debug, Default, Reflect)]
pub struct DropZone;

#[reflect_trait]
pub trait DropInteraction {
    fn can_accept(&self, droppable: Entity, world: &mut World) -> bool;
    fn on_enter(&self, droppable: Entity, world: &mut World);
    fn on_over(&self, droppable: Entity, world: &mut World);
    fn on_exit(&self, droppable: Entity, world: &mut World);
    fn on_drop(&self, droppable: Entity, world: &mut World);
}
