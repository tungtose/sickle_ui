use bevy::{ecs::system::EntityCommands, prelude::*};

use crate::ui_builder::UiBuilder;

use super::prelude::{MenuItem, MenuItemConfig, MenuItemUpdate, UiMenuItemExt};

pub struct ToggleMenuItemPlugin;

impl Plugin for ToggleMenuItemPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update_toggle_menu_item_value, update_toggle_menu_checkmark)
                .chain()
                .after(MenuItemUpdate),
        );
    }
}

fn update_toggle_menu_item_value(
    mut q_menu_items: Query<(&mut ToggleMenuItem, &MenuItem), Changed<MenuItem>>,
) {
    for (mut toggle, menu_item) in &mut q_menu_items {
        if menu_item.interacted() {
            toggle.checked = !toggle.checked;
        }
    }
}

fn update_toggle_menu_checkmark(
    mut q_menu_items: Query<(&ToggleMenuItem, &mut MenuItemConfig), Changed<ToggleMenuItem>>,
) {
    for (toggle, mut config) in &mut q_menu_items {
        if toggle.checked() {
            config.leading_icon = "sickle://icons/checkmark.png".to_string().into();
        } else {
            config.leading_icon = None;
        }
    }
}

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct ToggleMenuItem {
    checked: bool,
}

#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct ToggleMenuItemConfig {
    pub name: String,
    pub alt_code: Option<KeyCode>,
    pub shortcut: Option<Vec<KeyCode>>,
}

impl ToggleMenuItem {
    pub fn checked(&self) -> bool {
        self.checked
    }
}

impl Into<MenuItemConfig> for ToggleMenuItemConfig {
    fn into(self) -> MenuItemConfig {
        MenuItemConfig {
            name: self.name,
            alt_code: self.alt_code,
            shortcut: self.shortcut,
            ..default()
        }
    }
}

pub trait UiToggleMenuItemExt<'w, 's> {
    fn toggle_menu_item<'a>(
        &'a mut self,
        config: ToggleMenuItemConfig,
    ) -> EntityCommands<'w, 's, 'a>;
}

impl<'w, 's> UiToggleMenuItemExt<'w, 's> for UiBuilder<'w, 's, '_> {
    fn toggle_menu_item<'a>(
        &'a mut self,
        config: ToggleMenuItemConfig,
    ) -> EntityCommands<'w, 's, 'a> {
        let mut item = self.menu_item(config.clone().into());
        item.insert((ToggleMenuItem::default(), config));

        item
    }
}
