use bevy::prelude::*;

use crate::{
    ui_builder::*,
    ui_style::{SetImageExt, SetNodeHeightExt, SetNodeWidthExt, UiStyleExt},
};

pub struct IconPlugin;

impl Plugin for IconPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, pre_update_icon_config);
    }
}

fn pre_update_icon_config(
    q_icons: Query<(Entity, &IconConfig), (With<Icon>, Changed<IconConfig>)>,
    mut commands: Commands,
) {
    for (entity, config) in &q_icons {
        commands
            .style(entity)
            .width(config.width)
            .height(config.height)
            .image(config.path.clone());
    }
}

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct Icon;

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct IconConfig {
    pub width: Val,
    pub height: Val,
    pub path: String,
}

impl Default for IconConfig {
    fn default() -> Self {
        Self {
            width: Val::Px(16.),
            height: Val::Px(16.),
            path: "".into(),
        }
    }
}

pub trait UiIconExt<'w, 's> {
    fn icon<'a>(&'a mut self, config: IconConfig) -> UiBuilder<'w, 's, 'a>;
}

impl<'w, 's> UiIconExt<'w, 's> for UiBuilder<'w, 's, '_> {
    fn icon<'a>(&'a mut self, config: IconConfig) -> UiBuilder<'w, 's, 'a> {
        let path = config.path.clone();
        let mut icon = self.spawn((
            ImageBundle {
                style: Style {
                    width: config.width,
                    height: config.height,
                    ..default()
                },
                ..default()
            },
            Icon,
            config,
        ));

        icon.style().image(path);

        icon
    }
}
