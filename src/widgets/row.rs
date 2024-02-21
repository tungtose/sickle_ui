use bevy::prelude::*;

use crate::{
    ui_builder::*,
    ui_style::{SetBackgroundColorExt, SetNodeHeightExt},
};

use super::prelude::UiContainerExt;

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct Row;

#[derive(Debug, Default)]
pub struct RowConfig {
    pub height: Val,
    pub background_color: Color,
}

impl Row {
    fn frame() -> impl Bundle {
        NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        }
    }
}

pub trait UiRowExt<'w, 's> {
    fn row<'a>(
        &'a mut self,
        config: RowConfig,
        spawn_children: impl FnOnce(&mut UiBuilder),
    ) -> UiBuilder<'w, 's, 'a>;
}

impl<'w, 's> UiRowExt<'w, 's> for UiBuilder<'w, 's, '_> {
    fn row<'a>(
        &'a mut self,
        config: RowConfig,
        spawn_children: impl FnOnce(&mut UiBuilder),
    ) -> UiBuilder<'w, 's, 'a> {
        let mut row = self.container((Row::frame(), Row), spawn_children);

        row.style()
            .height(config.height)
            .background_color(config.background_color);

        row
    }
}
