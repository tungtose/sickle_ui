use bevy::{ecs::system::EntityCommands, prelude::*};

use crate::ui_builder::*;

use super::prelude::UiContainerExt;

#[derive(Debug, Default)]
pub struct ColumnConfig {
    pub width: Val,
    pub background_color: Color,
}

impl ColumnConfig {
    fn frame(&self) -> impl Bundle {
        NodeBundle {
            style: Style {
                width: self.width,
                height: Val::Percent(100.),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            background_color: self.background_color.into(),
            ..default()
        }
    }
}

pub trait UiColumnExt<'w, 's> {
    fn column<'a>(
        &'a mut self,
        config: ColumnConfig,
        spawn_children: impl FnOnce(&mut UiBuilder),
    ) -> EntityCommands<'w, 's, 'a>;
}

impl<'w, 's> UiColumnExt<'w, 's> for UiBuilder<'w, 's, '_> {
    fn column<'a>(
        &'a mut self,
        config: ColumnConfig,
        spawn_children: impl FnOnce(&mut UiBuilder),
    ) -> EntityCommands<'w, 's, 'a> {
        self.container(config.frame(), spawn_children)
    }
}
