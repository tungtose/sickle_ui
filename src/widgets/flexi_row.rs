use bevy::{ecs::system::EntityCommands, prelude::*};

use crate::{
    ui_builder::*,
    ui_commands::{SetBackgroundColorExt, SetBorderColorExt},
};

use super::prelude::UiContainerExt;

pub struct FlexiRowPlugin;

impl Plugin for FlexiRowPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_flexi_row_height);
    }
}

fn update_flexi_row_height(mut q_columns: Query<(&FlexiRow, &mut Style), Changed<FlexiRow>>) {
    for (config, mut style) in &mut q_columns {
        style.height = Val::Percent(config.height);
    }
}

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct FlexiRow {
    pub height: f32,
}

#[derive(Debug, Default)]
pub struct FlexiRowConfig {
    pub height: f32,
    pub background_color: Color,
    pub border_color: Color,
}

impl FlexiRow {
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

pub trait UiFlexiRowExt<'w, 's> {
    fn flexi_row<'a>(
        &'a mut self,
        config: FlexiRowConfig,
        spawn_children: impl FnOnce(&mut UiBuilder),
    ) -> EntityCommands<'w, 's, 'a>;
}

impl<'w, 's> UiFlexiRowExt<'w, 's> for UiBuilder<'w, 's, '_> {
    fn flexi_row<'a>(
        &'a mut self,
        config: FlexiRowConfig,
        spawn_children: impl FnOnce(&mut UiBuilder),
    ) -> EntityCommands<'w, 's, 'a> {
        let column = self
            .container(
                (
                    FlexiRow::frame(),
                    FlexiRow {
                        height: config.height,
                    },
                ),
                spawn_children,
            )
            .id();

        self.commands()
            .entity(column)
            .set_background_color(config.background_color)
            .set_border_color(config.border_color);

        self.commands().entity(column)
    }
}
