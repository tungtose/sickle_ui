use bevy::{
    ecs::{
        bundle::Bundle,
        entity::Entity,
        system::{Commands, EntityCommands},
    },
    hierarchy::BuildChildren,
};

pub struct UiBuilder<'w, 's, 'a> {
    commands: &'a mut Commands<'w, 's>,
    entity: Option<Entity>,
}

impl<'w, 's> UiBuilder<'w, 's, '_> {
    pub fn id(&self) -> Option<Entity> {
        self.entity
    }

    pub fn commands(&mut self) -> &mut Commands<'w, 's> {
        self.commands
    }

    pub fn entity_commands<'a>(&'a mut self) -> Result<EntityCommands<'w, 's, 'a>, &'static str> {
        if let Some(entity) = self.entity {
            Ok(self.commands().entity(entity))
        } else {
            Err("No entity set for UiBuilder")
        }
    }

    pub fn spawn<'a>(&'a mut self, bundle: impl Bundle) -> EntityCommands<'w, 's, 'a> {
        let mut new_entity = Entity::PLACEHOLDER;

        if let Some(entity) = self.id() {
            self.commands().entity(entity).with_children(|parent| {
                new_entity = parent.spawn(bundle).id();
            });
        } else {
            new_entity = self.commands().spawn(bundle).id();
        }

        self.commands().entity(new_entity)
    }
}

pub trait UiBuilderExt<'w, 's, 'a> {
    fn ui_builder(&'a mut self) -> UiBuilder<'w, 's, 'a>;
}

impl<'w, 's, 'a> UiBuilderExt<'w, 's, 'a> for EntityCommands<'w, 's, 'a> {
    fn ui_builder(&'a mut self) -> UiBuilder<'w, 's, 'a> {
        let entity = self.id();
        let mut builder = self.commands().ui_builder();
        builder.entity = entity.into();

        builder
    }
}

impl<'w, 's, 'a> UiBuilderExt<'w, 's, 'a> for Commands<'w, 's> {
    fn ui_builder(&'a mut self) -> UiBuilder<'w, 's, 'a> {
        UiBuilder::<'w, 's, 'a> {
            commands: self,
            entity: None,
        }
    }
}
