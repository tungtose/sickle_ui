use bevy::{ecs::system::EntityCommands, prelude::*};

use crate::{
    ui_builder::{UiBuilder, UiBuilderExt},
    ui_commands::SetEntityDisplayExt,
    FluxInteraction, FluxInteractionUpdate, TrackedInteraction,
};

use super::{
    menu::MenuUpdate,
    prelude::{MenuItemConfig, UiContainerExt, UiMenuItemExt},
};

const MENU_CONTAINER_Z_INDEX: i32 = 100001;

pub struct SubmenuPlugin;

impl Plugin for SubmenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            update_submenu_visiblity
                .after(FluxInteractionUpdate)
                .before(MenuUpdate),
        );
    }
}

fn update_submenu_visiblity(
    q_submenus: Query<(Entity, &Submenu, &FluxInteraction), Changed<FluxInteraction>>,
    q_containers: Query<&FluxInteraction>,
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
                .set_display(match entity == open {
                    true => Display::Flex,
                    false => Display::None,
                });
        }
    } else {
        for (_, submenu, _) in &q_submenus {
            let Ok(container_interaction) = q_containers.get(submenu.container) else {
                continue;
            };

            if *container_interaction == FluxInteraction::PointerEnter
                || *container_interaction == FluxInteraction::Pressed
            {
                commands
                    .entity(submenu.container)
                    .set_display(Display::Flex);
            } else {
                commands
                    .entity(submenu.container)
                    .set_display(Display::None);
            }
        }
    }
}

#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct SubmenuContainer;

#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct SubmenuConfig {
    pub name: String,
    pub alt_code: Option<KeyCode>,
    pub leading_icon: Option<String>,
}

impl Into<MenuItemConfig> for SubmenuConfig {
    fn into(self) -> MenuItemConfig {
        MenuItemConfig {
            name: self.name,
            alt_code: self.alt_code,
            leading_icon: self.leading_icon,
            trailing_icon: "sickle://icons/submenu.png".to_string().into(),
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
    fn menu_container() -> impl Bundle {
        (
            NodeBundle {
                style: Style {
                    left: Val::Percent(100.),
                    position_type: PositionType::Absolute,
                    border: UiRect::px(1., 1., 1., 1.),
                    padding: UiRect::px(5., 5., 5., 10.),
                    margin: UiRect::px(-5., 0., -5., 0.),
                    flex_direction: FlexDirection::Column,
                    align_self: AlignSelf::FlexStart,
                    align_items: AlignItems::Stretch,
                    display: Display::None,
                    ..default()
                },
                z_index: ZIndex::Global(MENU_CONTAINER_Z_INDEX),
                background_color: Color::rgb(0.7, 0.6, 0.5).into(),
                border_color: Color::WHITE.into(),
                ..default()
            },
            Interaction::default(),
            TrackedInteraction::default(),
            SubmenuContainer,
        )
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
        let menu_id = self.menu_item(config.clone().into()).id();
        let container = self
            .commands()
            .entity(menu_id)
            .ui_builder()
            .container(Submenu::menu_container(), spawn_items)
            .id();

        self.commands()
            .entity(menu_id)
            .insert((Submenu { container }, config));

        self.commands().entity(menu_id)
    }
}
