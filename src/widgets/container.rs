use bevy::prelude::*;

use crate::ui_builder::*;

pub trait UiContainerExt<'w, 's> {
    fn container<'a>(
        &'a mut self,
        bundle: impl Bundle,
        spawn_children: impl FnOnce(&mut UiBuilder),
    ) -> UiBuilder<'w, 's, 'a>;
}

impl<'w, 's> UiContainerExt<'w, 's> for UiBuilder<'w, 's, '_> {
    fn container<'a>(
        &'a mut self,
        bundle: impl Bundle,
        spawn_children: impl FnOnce(&mut UiBuilder),
    ) -> UiBuilder<'w, 's, 'a> {
        let mut new_builder = self.spawn(bundle);
        spawn_children(&mut new_builder);

        new_builder
    }
}
