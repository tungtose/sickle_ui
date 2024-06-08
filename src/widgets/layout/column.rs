use bevy::prelude::*;

use sickle_ui_scaffold::{
    ui_builder::{UiBuilder, UiRoot},
    ui_style::prelude::*,
};

use super::container::UiContainerExt;

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct Column;

impl Column {
    fn frame() -> impl Bundle {
        (
            Name::new("Column"),
            NodeBundle {
                style: Style {
                    height: Val::Percent(100.),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                background_color: Color::NONE.into(),
                ..default()
            },
            LockedStyleAttributes::lock(LockableStyleAttribute::FlexDirection),
        )
    }
}

pub trait UiColumnExt<'w, 's> {
    fn column<'a>(
        &'a mut self,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<'w, 's, 'a, Entity>;
}

impl<'w, 's> UiColumnExt<'w, 's> for UiBuilder<'w, 's, '_, UiRoot> {
    fn column<'a>(
        &'a mut self,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<'w, 's, 'a, Entity> {
        self.container((Column::frame(), Column), spawn_children)
    }
}

impl<'w, 's> UiColumnExt<'w, 's> for UiBuilder<'w, 's, '_, Entity> {
    fn column<'a>(
        &'a mut self,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<'w, 's, 'a, Entity> {
        self.container((Column::frame(), Column), spawn_children)
    }
}