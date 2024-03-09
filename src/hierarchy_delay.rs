use bevy::{ecs::system::EntityCommands, prelude::*};

use crate::ui_style::{SetNodeShowHideExt, UiStyleExt};

pub struct HierarchyDelayPlugin;

impl Plugin for HierarchyDelayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (process_delayed_reparents, process_delayed_despawns)
                .chain()
                .in_set(HierarchyDelayPreUpdate),
        );
    }
}

#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct HierarchyDelayPreUpdate;

/// Workaround for a misconception with the taffy integration:
///
/// If an entity is re-parented during the creation of the new parent
/// (see e.g. PopoutPanelFromTabContainer), despawning the new parent later
/// causes a panic in taffy as the parent/children map is not in sync with Bevy
///
/// This is probably because the commands reserve an Entity ID during addition,
/// but it is not evident that the new Entity will be a Node taffy needs to care about.
/// Might be a bug also, but unlikely.
///
/// TODO: Report that docs need to highlight this
fn process_delayed_reparents(
    mut q_reparent: Query<(Entity, &DelayedReparent), Added<DelayedReparent>>,
    q_show_comp: Query<(&Style, &Visibility)>,
    mut commands: Commands,
) {
    for (entity, to_reparent) in &mut q_reparent {
        commands
            .entity(entity)
            .set_parent(to_reparent.new_parent)
            .remove::<DelayedReparent>();

        if let Ok(_) = q_show_comp.get(entity) {
            commands.style(entity).show();
        }
    }
}

/// Workaround for an issue delayed reparenting:
///
/// Since an entity will be reparented in the next frame, despawning the parent
/// is not possible immediately. DelayedDespawn will be processed AFTER
/// DelayedReparenting, making sure the entity is removed before the parent is
/// despawned.
fn process_delayed_despawns(
    q_to_despawn: Query<(Entity, &DelayedDespawn), Added<DelayedDespawn>>,
    mut commands: Commands,
) {
    for (entity, to_despawn) in &q_to_despawn {
        if to_despawn.recursive {
            commands.entity(entity).despawn_recursive();
        } else {
            commands.entity(entity).despawn();
        }
    }
}

#[derive(Component)]
#[component(storage = "SparseSet")]
struct DelayedReparent {
    new_parent: Entity,
}

#[derive(Component)]
#[component(storage = "SparseSet")]
struct DelayedDespawn {
    recursive: bool,
}

pub trait DelayActions<'a> {
    fn delay_reparenting(&'a mut self, new_parent: Entity) -> &mut EntityCommands<'a>;
    fn delay_depsawn(&'a mut self, recursive: bool) -> &mut EntityCommands<'a>;
}

impl<'a> DelayActions<'a> for EntityCommands<'a> {
    fn delay_reparenting(&'a mut self, new_parent: Entity) -> &mut EntityCommands<'a> {
        self.insert(DelayedReparent { new_parent });

        self
    }

    fn delay_depsawn(&'a mut self, recursive: bool) -> &mut EntityCommands<'a> {
        self.insert(DelayedDespawn { recursive });

        self
    }
}
