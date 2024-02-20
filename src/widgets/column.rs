use bevy::prelude::*;

use crate::{
    ui_builder::*,
    ui_style::{SetBackgroundColorExt, SetEntityWidthExt},
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
    ) -> UiBuilder<'w, 's, 'a>;
}

impl<'w, 's> UiColumnExt<'w, 's> for UiBuilder<'w, 's, '_> {
    fn column<'a>(
        &'a mut self,
        config: ColumnConfig,
        spawn_children: impl FnOnce(&mut UiBuilder),
    ) -> UiBuilder<'w, 's, 'a> {
        let mut column = self.container((Column::frame(), Column), spawn_children);

        column
            .style()
            .width(config.width)
            .background_color(config.background_color);

        column
    }
}
