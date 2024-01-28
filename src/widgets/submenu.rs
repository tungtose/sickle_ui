use bevy::{ecs::system::EntityCommands, prelude::*};

use crate::{
    ui_builder::UiBuilder, ui_commands::SetEntityDisplayExt, FluxInteraction, FluxInteractionUpdate,
};

use super::prelude::{MenuItemConfig, UiContainerExt, UiMenuItemExt};

const MENU_CONTAINER_Z_INDEX: i32 = 100000;

pub struct SubmenuPlugin;

impl Plugin for SubmenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            update_submenu_visiblity.after(FluxInteractionUpdate),
        );
    }
}

fn update_submenu_visiblity(
    q_submenus: Query<(Entity, &Submenu, &FluxInteraction), Changed<FluxInteraction>>,
    mut commands: Commands,
) {
    let mut open: Option<Entity> = None;

    for (entity, _, interaction) in &q_submenus {
        if *interaction == FluxInteraction::PointerEnter {
            open = entity.into();
            break;
        }
    }

    if let Some(open) = open {
        for (entity, submenu, _) in &q_submenus {
            commands
                .entity(submenu.container)
                .set_display(if entity == open {
                    Display::Flex
                } else {
                    Display::None
                });
        }
    } else {
        for (_, submenu, _) in &q_submenus {
            commands
                .entity(submenu.container)
                .set_display(Display::None);
        }
    }
}

#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct SubmenuConfig {
    pub name: String,
    pub alt_code: Option<KeyCode>,
    pub leading_icon: Option<Handle<Image>>,
}

impl Into<MenuItemConfig> for SubmenuConfig {
    fn into(self) -> MenuItemConfig {
        MenuItemConfig {
            name: self.name,
            alt_code: self.alt_code,
            leading_icon: self.leading_icon,
            ..default()
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct Submenu {
    container: Entity,
}

impl Default for Submenu {
    fn default() -> Self {
        Self {
            container: Entity::PLACEHOLDER,
        }
    }
}

impl Submenu {
    fn wrapper() -> impl Bundle {
        (
            NodeBundle {
                focus_policy: bevy::ui::FocusPolicy::Block,
                ..default()
            },
            Interaction::default(),
            FluxInteraction::default(),
        )
    }

    fn menu_container() -> impl Bundle {
        NodeBundle {
            style: Style {
                left: Val::Percent(100.),
                position_type: PositionType::Absolute,
                border: UiRect::px(1., 1., 0., 1.),
                padding: UiRect::px(5., 5., 5., 10.),
                flex_direction: FlexDirection::Column,
                align_self: AlignSelf::End,
                align_items: AlignItems::Stretch,
                display: Display::None,
                ..default()
            },
            z_index: ZIndex::Global(MENU_CONTAINER_Z_INDEX),
            background_color: Color::rgb(0.7, 0.6, 0.5).into(),
            border_color: Color::WHITE.into(),
            ..default()
        }
    }
}

pub trait UiSubmenuExt<'w, 's> {
    fn submenu<'a>(
        &'a mut self,
        config: SubmenuConfig,
        spawn_items: impl FnOnce(&mut UiBuilder),
    ) -> EntityCommands<'w, 's, 'a>;
}

impl<'w, 's> UiSubmenuExt<'w, 's> for UiBuilder<'w, 's, '_> {
    fn submenu<'a>(
        &'a mut self,
        config: SubmenuConfig,
        spawn_items: impl FnOnce(&mut UiBuilder),
    ) -> EntityCommands<'w, 's, 'a> {
        let mut container = Entity::PLACEHOLDER;
        let mut wrapper = self.container(Submenu::wrapper(), |wrapper| {
            wrapper.menu_item(config.clone().into());
            container = wrapper
                .container(Submenu::menu_container(), spawn_items)
                .id();
        });

        wrapper.insert((Submenu { container }, config));

        wrapper
    }
}
