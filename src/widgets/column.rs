use bevy::{ecs::system::EntityCommands, prelude::*};

use crate::ui_builder::*;

#[derive(Debug, Default)]
pub struct ColumnConfig {
    pub width: Val,
    pub background_color: Color,
}

pub trait UiColumnExt<'w, 's> {
    fn column<'a>(
        &'a mut self,
        config: ColumnConfig,
        f: impl FnOnce(&mut UiBuilder),
    ) -> EntityCommands<'w, 's, 'a>;
}

impl<'w, 's> UiColumnExt<'w, 's> for UiBuilder<'w, 's, '_> {
    fn column<'a>(
        &'a mut self,
        config: ColumnConfig,
        spawn_children: impl FnOnce(&mut UiBuilder),
    ) -> EntityCommands<'w, 's, 'a> {
        let mut new_parent = Entity::PLACEHOLDER;

        if let Some(entity) = self.entity() {
            self.commands().entity(entity).with_children(|parent| {
                new_parent = parent.spawn(column_bundle(config)).id();
            });
        } else {
            new_parent = self.commands().spawn(column_bundle(config)).id();
        }

        let mut new_entity = self.commands().entity(new_parent);
        let mut new_builder = new_entity.ui_builder();
        spawn_children(&mut new_builder);

        self.commands().entity(new_parent)
    }
}

fn column_bundle(config: ColumnConfig) -> impl Bundle {
    NodeBundle {
        style: Style {
            height: Val::Percent(100.),
            width: config.width,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        background_color: config.background_color.into(),
        ..default()
    }
}
