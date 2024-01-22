use bevy::{ecs::system::EntityCommands, prelude::*};

use crate::ui_builder::*;

use super::prelude::UiContainerExt;

#[derive(Debug, Default)]
pub struct RowConfig {
    pub height: Val,
    pub background_color: Color,
}

impl RowConfig {
    fn frame(&self) -> impl Bundle {
        NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: self.height,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: self.background_color.into(),
            ..default()
        }
    }
}

pub trait UiRowExt<'w, 's> {
    fn row<'a>(
        &'a mut self,
        config: RowConfig,
        spawn_children: impl FnOnce(&mut UiBuilder),
    ) -> EntityCommands<'w, 's, 'a>;
}

impl<'w, 's> UiRowExt<'w, 's> for UiBuilder<'w, 's, '_> {
    fn row<'a>(
        &'a mut self,
        config: RowConfig,
        spawn_children: impl FnOnce(&mut UiBuilder),
    ) -> EntityCommands<'w, 's, 'a> {
        self.container(config.frame(), spawn_children)
    }
}
