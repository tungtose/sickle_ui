use bevy::prelude::*;

use crate::ui_builder::UiBuilder;

use super::prelude::UiContainerExt;

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct Panel {
    title: String,
}

impl Default for Panel {
    fn default() -> Self {
        Self { title: "".into() }
    }
}

impl Panel {
    pub fn title(&self) -> String {
        self.title.clone()
    }

    fn frame() -> impl Bundle {
        NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ..default()
        }
    }
}

pub trait UiPanelExt<'w, 's> {
    fn panel<'a>(
        &'a mut self,
        title: String,
        spawn_children: impl FnOnce(&mut UiBuilder),
    ) -> UiBuilder<'w, 's, 'a>;
}

impl<'w, 's> UiPanelExt<'w, 's> for UiBuilder<'w, 's, '_> {
    fn panel<'a>(
        &'a mut self,
        title: String,
        spawn_children: impl FnOnce(&mut UiBuilder),
    ) -> UiBuilder<'w, 's, 'a> {
        self.container((Panel::frame(), Panel { title }), spawn_children)
    }
}
