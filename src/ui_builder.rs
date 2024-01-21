use bevy::ecs::{
    entity::Entity,
    system::{Commands, EntityCommands},
};

pub struct UiBuilder<'w, 's, 'a> {
    commands: &'a mut Commands<'w, 's>,
    entity: Option<Entity>,
}

impl<'w, 's> UiBuilder<'w, 's, '_> {
    pub fn entity(&self) -> Option<Entity> {
        self.entity
    }

    pub fn commands(&mut self) -> &mut Commands<'w, 's> {
        self.commands
    }

    pub fn entity_commands<'a>(&'a mut self) -> Result<EntityCommands<'w, 's, 'a>, &'static str> {
        if let Some(entity) = self.entity {
            Ok(self.commands().entity(entity))
        } else {
            Err("No entity set for UiBuilder")
        }
    }
}

pub trait UiBuilderExt<'w, 's, 'a> {
    fn ui_builder(&'a mut self) -> UiBuilder<'w, 's, 'a>;
}

impl<'w, 's, 'a> UiBuilderExt<'w, 's, 'a> for EntityCommands<'w, 's, 'a> {
    fn ui_builder(&'a mut self) -> UiBuilder<'w, 's, 'a> {
        let entity = self.id();
        let mut builder = self.commands().ui_builder();
        builder.entity = entity.into();

        builder
    }
}

impl<'w, 's, 'a> UiBuilderExt<'w, 's, 'a> for Commands<'w, 's> {
    fn ui_builder(&'a mut self) -> UiBuilder<'w, 's, 'a> {
        UiBuilder::<'w, 's, 'a> {
            commands: self,
            entity: None,
        }
    }
}

// TODO: Replace strings with impl Into::<String>

/*

pub struct MenuConfig{
    pub name: String,
    pub alt_code: Option<KeyCode>,
}

pub struct MenuItemConfig{
    pub name: String,
    pub icon: Option<Handle<Image>>,
    pub alt_code: Option<KeyCode>,
    pub shortcut: Option<Vec<KeyCode>>,
}

commands.ui_builder().menu(MenuConfig{...}, |menu|{
    menu.menu_item(MenuItemConfig{...}).insert(MyMenu::ItemOne);
    menu.menu_item(MenuItemConfig{...}).insert(MyMenu::ItemTwo);
    menu.menu_item(MenuItemConfig{...}).insert(MyMenu::ItemThree);
    menu.sub_menu(MenuConfig{...}, |sub_menu|{
        sub_menu.menu_item(MenuItemConfig{...}).insert(MyMenu::ItemOne);
        sub_menu.menu_item(MenuItemConfig{...}).insert(MyMenu::ItemTwo);
        sub_menu.menu_item(MenuItemConfig{...}).insert(MyMenu::ItemThree);
    });
})

*/
