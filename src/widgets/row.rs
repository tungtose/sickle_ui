use bevy::prelude::*;

use crate::ui_builder::*;

use super::prelude::UiContainerExt;

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct Row;

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
    fn row<'a>(&'a mut self, spawn_children: impl FnOnce(&mut UiBuilder)) -> UiBuilder<'w, 's, 'a>;
}

impl<'w, 's> UiRowExt<'w, 's> for UiBuilder<'w, 's, '_> {
    fn row<'a>(&'a mut self, spawn_children: impl FnOnce(&mut UiBuilder)) -> UiBuilder<'w, 's, 'a> {
        self.container((Row::frame(), Row), spawn_children)
    }
}
