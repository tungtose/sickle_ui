use bevy::{ecs::system::EntityCommands, prelude::*};

use crate::{
    ui_builder::*,
    ui_commands::{SetBackgroundColorExt, SetEntityWidthExt},
};

use super::prelude::UiContainerExt;

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct Column;

#[derive(Debug, Default)]
pub struct ColumnConfig {
    pub width: Val,
    pub background_color: Color,
}

impl Column {
    fn frame() -> impl Bundle {
        NodeBundle {
            style: Style {
                height: Val::Percent(100.),
                flex_direction: FlexDirection::Column,
                ..default()
            },
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
        let column = self
            .container((Column::frame(), Column), spawn_children)
            .id();

        self.commands()
            .entity(column)
            .set_width(config.width)
            .set_background_color(config.background_color);

        self.commands().entity(column)
    }
}
