use bevy::prelude::*;

pub struct HierarchyPlugin;

impl Plugin for HierarchyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, move_node_to_parent);
    }
}

fn move_node_to_parent(
    q_to_move: Query<(Entity, &MoveToParent), Added<MoveToParent>>,
    mut commands: Commands,
) {
    for (entity, to_move) in &q_to_move {
        //println!("Moving {:?} to {:?}", entity, to_move.parent);

        if let Some(parent) = to_move.parent {
            commands
                .entity(entity)
                .set_parent(parent)
                .remove::<MoveToParent>();
        } else {
            commands
                .entity(entity)
                .remove_parent()
                .remove::<MoveToParent>();
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[component(storage = "SparseSet")]
#[reflect(Component)]
pub struct MoveToParent {
    pub parent: Option<Entity>,
}

impl Default for MoveToParent {
    fn default() -> Self {
        Self { parent: None }
    }
}
