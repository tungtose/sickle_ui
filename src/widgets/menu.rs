use bevy::{ecs::system::EntityCommands, prelude::*};
use sickle_math::ease::Ease;

use crate::{
    animated_interaction::{AnimatedInteraction, AnimationConfig},
    interactions::InteractiveBackground,
    ui_builder::*,
    FluxInteraction, TrackedInteraction,
};

use super::prelude::{LabelConfig, UiContainerExt, UiLabelExt};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_menu_press);
    }
}

fn handle_menu_press(q_menus: Query<(&Menu, &FluxInteraction), Changed<FluxInteraction>>) {
    for (menu, interaction) in &q_menus {
        if *interaction != FluxInteraction::Pressed {
            continue;
        }

        println!("Menu pressed: {:?}", menu);
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct Menu {
    container: Entity,
}

impl Default for Menu {
    fn default() -> Self {
        Self {
            container: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Debug, Default)]
pub struct MenuConfig {
    pub name: String,
    pub alt_code: Option<KeyCode>,
    //pub icon: Option<Handle<Image>>,
}

#[derive(Debug, Default)]
pub struct MenuItemConfig {
    pub name: String,
    //pub icon: Option<Handle<Image>>,
    pub alt_code: Option<KeyCode>,
    pub shortcut: Option<Vec<KeyCode>>,
}

impl Menu {
    fn base_tween() -> AnimationConfig {
        AnimationConfig {
            duration: 0.1,
            easing: Ease::OutExpo,
            ..default()
        }
    }

    fn button() -> impl Bundle {
        (
            ButtonBundle {
                style: Style {
                    padding: UiRect::axes(Val::Px(10.), Val::Px(5.)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: Color::NONE.into(),
                ..default()
            },
            TrackedInteraction::default(),
            InteractiveBackground {
                highlight: Some(Color::rgba(9., 8., 7., 0.5)),
                ..default()
            },
            AnimatedInteraction::<InteractiveBackground> {
                tween: Menu::base_tween(),
                ..default()
            },
        )
    }

    fn container() -> impl Bundle {
        NodeBundle {
            style: Style {
                border: UiRect::px(1., 1., 0., 1.),
                flex_direction: FlexDirection::Column,
                display: Display::None,
                ..default()
            },
            background_color: Color::BEIGE.into(),
            border_color: Color::WHITE.into(),
            ..default()
        }
    }
}

pub trait UiMenuExt<'w, 's> {
    fn menu<'a>(
        &'a mut self,
        config: MenuConfig,
        spawn_items: impl FnOnce(&mut UiBuilder),
    ) -> EntityCommands<'w, 's, 'a>;
}

impl<'w, 's> UiMenuExt<'w, 's> for UiBuilder<'w, 's, '_> {
    fn menu<'a>(
        &'a mut self,
        config: MenuConfig,
        spawn_items: impl FnOnce(&mut UiBuilder),
    ) -> EntityCommands<'w, 's, 'a> {
        let mut container = Entity::PLACEHOLDER;
        let mut menu = self.container(Menu::button(), |menu_button| {
            menu_button.label(LabelConfig {
                label: config.name,
                ..default()
            });
            container = menu_button.container(Menu::container(), spawn_items).id();
        });

        menu.insert(Menu { container });

        menu
    }
}
