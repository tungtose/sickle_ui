use bevy::{
    asset::AssetServer,
    ecs::{
        entity::Entity,
        system::{Command, EntityCommands},
        world::World,
    },
    text::{Text, TextSection, TextStyle},
    ui::{Display, Style, UiImage},
};

struct SetEntityDisplay {
    entity: Entity,
    display: Display,
}

impl Command for SetEntityDisplay {
    fn apply(self, world: &mut World) {
        let mut q_style = world.query::<&mut Style>();
        let Ok(mut style) = q_style.get_mut(world, self.entity) else {
            return;
        };

        if style.display != self.display {
            style.display = self.display;
        }
    }
}

pub trait SetEntityDisplayExt<'w, 's, 'a> {
    fn set_display(&'a mut self, display: Display) -> EntityCommands<'w, 's, 'a>;
}

impl<'w, 's, 'a> SetEntityDisplayExt<'w, 's, 'a> for EntityCommands<'w, 's, 'a> {
    fn set_display(&'a mut self, display: Display) -> EntityCommands<'w, 's, 'a> {
        let entity = self.id();
        self.commands().add(SetEntityDisplay { entity, display });

        self.commands().entity(entity)
    }
}

struct SetTextSections {
    entity: Entity,
    sections: Vec<TextSection>,
}

impl Command for SetTextSections {
    fn apply(self, world: &mut World) {
        let mut q_text = world.query::<&mut Text>();
        let Ok(mut text) = q_text.get_mut(world, self.entity) else {
            return;
        };

        text.sections = self.sections;
    }
}

pub trait SetTextSectionsExt<'w, 's, 'a> {
    fn set_text_sections(&'a mut self, sections: Vec<TextSection>) -> EntityCommands<'w, 's, 'a>;
}

impl<'w, 's, 'a> SetTextSectionsExt<'w, 's, 'a> for EntityCommands<'w, 's, 'a> {
    fn set_text_sections(&'a mut self, sections: Vec<TextSection>) -> EntityCommands<'w, 's, 'a> {
        let entity = self.id();
        self.commands().add(SetTextSections { entity, sections });

        self.commands().entity(entity)
    }
}

struct SetText {
    entity: Entity,
    text: String,
    style: TextStyle,
}

impl Command for SetText {
    fn apply(self, world: &mut World) {
        let mut q_text = world.query::<&mut Text>();
        let Ok(mut text) = q_text.get_mut(world, self.entity) else {
            return;
        };

        text.sections = vec![TextSection::new(self.text, self.style)];
    }
}

pub trait SetTextExt<'w, 's, 'a> {
    fn set_text(
        &'a mut self,
        text: impl Into<String>,
        style: Option<TextStyle>,
    ) -> EntityCommands<'w, 's, 'a>;
}

impl<'w, 's, 'a> SetTextExt<'w, 's, 'a> for EntityCommands<'w, 's, 'a> {
    fn set_text(
        &'a mut self,
        text: impl Into<String>,
        style: Option<TextStyle>,
    ) -> EntityCommands<'w, 's, 'a> {
        let entity = self.id();
        self.commands().add(SetText {
            entity,
            text: text.into(),
            style: style.unwrap_or_default(),
        });

        self.commands().entity(entity)
    }
}

struct SetImage {
    entity: Entity,
    path: String,
}

impl Command for SetImage {
    fn apply(self, world: &mut World) {
        let handle = world.resource::<AssetServer>().load(self.path);
        let mut q_ui_image = world.query::<&mut UiImage>();
        let Ok(mut image) = q_ui_image.get_mut(world, self.entity) else {
            return;
        };
        image.texture = handle;
    }
}

pub trait SetImageExt<'w, 's, 'a> {
    fn set_image(&'a mut self, path: impl Into<String>) -> EntityCommands<'w, 's, 'a>;
}

impl<'w, 's, 'a> SetImageExt<'w, 's, 'a> for EntityCommands<'w, 's, 'a> {
    fn set_image(&'a mut self, path: impl Into<String>) -> EntityCommands<'w, 's, 'a> {
        let entity = self.id();
        self.commands().add(SetImage {
            entity,
            path: path.into(),
        });

        self.commands().entity(entity)
    }
}
