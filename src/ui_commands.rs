use bevy::{
    asset::{AssetServer, Handle},
    ecs::{
        entity::Entity,
        system::{Command, EntityCommands},
        world::World,
    },
    log::warn,
    render::color::Color,
    text::{Text, TextSection, TextStyle},
    ui::{BackgroundColor, BorderColor, Display, Style, UiImage},
};

struct SetEntityDisplay {
    entity: Entity,
    display: Display,
}

impl Command for SetEntityDisplay {
    fn apply(self, world: &mut World) {
        let mut q_style = world.query::<&mut Style>();
        let Ok(mut style) = q_style.get_mut(world, self.entity) else {
            warn!(
                "Failed to set display property on entity {:?}: No Style component found!",
                self.entity
            );
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
            warn!(
                "Failed to set text sections on entity {:?}: No Text component found!",
                self.entity
            );
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
            warn!(
                "Failed to set text on entity {:?}: No Text component found!",
                self.entity
            );
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
        let handle = if self.path == "" {
            Handle::default()
        } else {
            world.resource::<AssetServer>().load(self.path)
        };

        let mut q_ui_image = world.query::<&mut UiImage>();
        let Ok(mut image) = q_ui_image.get_mut(world, self.entity) else {
            warn!(
                "Failed to set image on entity {:?}: No UiImage component found!",
                self.entity
            );
            return;
        };

        if image.texture != handle {
            image.texture = handle;
        }
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

struct SetBorderColor {
    entity: Entity,
    color: Color,
}

impl Command for SetBorderColor {
    fn apply(self, world: &mut World) {
        let mut q_border_color = world.query::<&mut BorderColor>();
        let Ok(mut border_color) = q_border_color.get_mut(world, self.entity) else {
            warn!(
                "Failed to set border color on entity {:?}: No BorderColor component found!",
                self.entity
            );
            return;
        };

        if border_color.0 != self.color.into() {
            border_color.0 = self.color.into();
        }
    }
}

pub trait SetBorderColorExt<'w, 's, 'a> {
    fn set_border_color(&'a mut self, color: Color) -> EntityCommands<'w, 's, 'a>;
}

impl<'w, 's, 'a> SetBorderColorExt<'w, 's, 'a> for EntityCommands<'w, 's, 'a> {
    fn set_border_color(&'a mut self, color: Color) -> EntityCommands<'w, 's, 'a> {
        let entity = self.id();
        self.commands().add(SetBorderColor { entity, color });

        self.commands().entity(entity)
    }
}

struct SetBackgroundColor {
    entity: Entity,
    color: Color,
}

impl Command for SetBackgroundColor {
    fn apply(self, world: &mut World) {
        let mut q_background_color = world.query::<&mut BackgroundColor>();
        let Ok(mut background_color) = q_background_color.get_mut(world, self.entity) else {
            warn!(
                "Failed to set background color on entity {:?}: No BackgroundColor component found!",
                self.entity
            );
            return;
        };

        if background_color.0 != self.color.into() {
            background_color.0 = self.color.into();
        }
    }
}

pub trait SetBackgroundColorExt<'w, 's, 'a> {
    fn set_background_color(&'a mut self, color: Color) -> EntityCommands<'w, 's, 'a>;
}

impl<'w, 's, 'a> SetBackgroundColorExt<'w, 's, 'a> for EntityCommands<'w, 's, 'a> {
    fn set_background_color(&'a mut self, color: Color) -> EntityCommands<'w, 's, 'a> {
        let entity = self.id();
        self.commands().add(SetBackgroundColor { entity, color });

        self.commands().entity(entity)
    }
}
