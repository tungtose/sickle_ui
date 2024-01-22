use bevy::{ecs::system::EntityCommands, prelude::*};

use crate::ui_builder::*;

pub trait UiContainerExt<'w, 's> {
    fn container<'a>(
        &'a mut self,
        bundle: impl Bundle,
        spawn_children: impl FnOnce(&mut UiBuilder),
    ) -> EntityCommands<'w, 's, 'a>;
}

impl<'w, 's> UiContainerExt<'w, 's> for UiBuilder<'w, 's, '_> {
    fn container<'a>(
        &'a mut self,
        bundle: impl Bundle,
        spawn_children: impl FnOnce(&mut UiBuilder),
    ) -> EntityCommands<'w, 's, 'a> {
        let mut new_entity = self.spawn(bundle);
        let new_entity_id = new_entity.id();

        let mut new_builder = new_entity.ui_builder();
        spawn_children(&mut new_builder);

        self.commands().entity(new_entity_id)
    }
}
