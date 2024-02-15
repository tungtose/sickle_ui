use bevy::{
    asset::{AssetServer, Handle},
    ecs::{
        entity::Entity,
        query::With,
        system::{Command, Commands, EntityCommands},
        world::World,
    },
    log::warn,
    render::{color::Color, view::Visibility},
    text::{Text, TextSection, TextStyle},
    ui::{AlignSelf, BackgroundColor, BorderColor, Display, JustifySelf, Style, UiImage, Val},
    window::{CursorIcon, PrimaryWindow, Window},
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

struct SetEntityVisiblity {
    entity: Entity,
    visibility: Visibility,
}

impl Command for SetEntityVisiblity {
    fn apply(self, world: &mut World) {
        let mut q_visibility = world.query::<&mut Visibility>();
        let Ok(mut visiblity) = q_visibility.get_mut(world, self.entity) else {
            warn!(
                "Failed to set visiblity on entity {:?}: No Visibility component found!",
                self.entity
            );
            return;
        };

        if *visiblity != self.visibility {
            *visiblity = self.visibility;
        }
    }
}

pub trait SetEntityVisiblityExt<'w, 's, 'a> {
    fn set_visibility(&'a mut self, visibility: Visibility) -> EntityCommands<'w, 's, 'a>;
}

impl<'w, 's, 'a> SetEntityVisiblityExt<'w, 's, 'a> for EntityCommands<'w, 's, 'a> {
    fn set_visibility(&'a mut self, visibility: Visibility) -> EntityCommands<'w, 's, 'a> {
        let entity = self.id();
        self.commands()
            .add(SetEntityVisiblity { entity, visibility });

        self.commands().entity(entity)
    }
}

struct SetEntityWidth {
    entity: Entity,
    width: Val,
}

impl Command for SetEntityWidth {
    fn apply(self, world: &mut World) {
        let mut q_style = world.query::<&mut Style>();
        let Ok(mut style) = q_style.get_mut(world, self.entity) else {
            warn!(
                "Failed to set width property on entity {:?}: No Style component found!",
                self.entity
            );
            return;
        };

        if style.width != self.width {
            style.width = self.width;
        }
    }
}

pub trait SetEntityWidthExt<'w, 's, 'a> {
    fn set_width(&'a mut self, width: Val) -> EntityCommands<'w, 's, 'a>;
}

impl<'w, 's, 'a> SetEntityWidthExt<'w, 's, 'a> for EntityCommands<'w, 's, 'a> {
    fn set_width(&'a mut self, width: Val) -> EntityCommands<'w, 's, 'a> {
        let entity = self.id();
        self.commands().add(SetEntityWidth { entity, width });

        self.commands().entity(entity)
    }
}

struct SetEntityHeight {
    entity: Entity,
    height: Val,
}

impl Command for SetEntityHeight {
    fn apply(self, world: &mut World) {
        let mut q_style = world.query::<&mut Style>();
        let Ok(mut style) = q_style.get_mut(world, self.entity) else {
            warn!(
                "Failed to set height property on entity {:?}: No Style component found!",
                self.entity
            );
            return;
        };

        if style.height != self.height {
            style.height = self.height;
        }
    }
}

pub trait SetEntityHeightExt<'w, 's, 'a> {
    fn set_height(&'a mut self, height: Val) -> EntityCommands<'w, 's, 'a>;
}

impl<'w, 's, 'a> SetEntityHeightExt<'w, 's, 'a> for EntityCommands<'w, 's, 'a> {
    fn set_height(&'a mut self, height: Val) -> EntityCommands<'w, 's, 'a> {
        let entity = self.id();
        self.commands().add(SetEntityHeight { entity, height });

        self.commands().entity(entity)
    }
}

struct SetEntityAlignSelf {
    entity: Entity,
    align_self: AlignSelf,
}

impl Command for SetEntityAlignSelf {
    fn apply(self, world: &mut World) {
        let mut q_style = world.query::<&mut Style>();
        let Ok(mut style) = q_style.get_mut(world, self.entity) else {
            warn!(
                "Failed to set align self property on entity {:?}: No Style component found!",
                self.entity
            );
            return;
        };

        if style.align_self != self.align_self {
            style.align_self = self.align_self;
        }
    }
}

pub trait SetEntityAlignSelfExt<'w, 's, 'a> {
    fn align_self(&'a mut self, align_self: AlignSelf) -> EntityCommands<'w, 's, 'a>;
}

impl<'w, 's, 'a> SetEntityAlignSelfExt<'w, 's, 'a> for EntityCommands<'w, 's, 'a> {
    fn align_self(&'a mut self, align_self: AlignSelf) -> EntityCommands<'w, 's, 'a> {
        let entity = self.id();
        self.commands()
            .add(SetEntityAlignSelf { entity, align_self });

        self.commands().entity(entity)
    }
}

struct SetEntityJustifySelf {
    entity: Entity,
    justify_self: JustifySelf,
}

impl Command for SetEntityJustifySelf {
    fn apply(self, world: &mut World) {
        let mut q_style = world.query::<&mut Style>();
        let Ok(mut style) = q_style.get_mut(world, self.entity) else {
            warn!(
                "Failed to set justify self property on entity {:?}: No Style component found!",
                self.entity
            );
            return;
        };

        if style.justify_self != self.justify_self {
            style.justify_self = self.justify_self;
        }
    }
}

pub trait SetEntityJustifySelfExt<'w, 's, 'a> {
    fn justify_self(&'a mut self, justify_self: JustifySelf) -> EntityCommands<'w, 's, 'a>;
}

impl<'w, 's, 'a> SetEntityJustifySelfExt<'w, 's, 'a> for EntityCommands<'w, 's, 'a> {
    fn justify_self(&'a mut self, justify_self: JustifySelf) -> EntityCommands<'w, 's, 'a> {
        let entity = self.id();
        self.commands().add(SetEntityJustifySelf {
            entity,
            justify_self,
        });

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

struct SetCursor {
    cursor: CursorIcon,
}

impl Command for SetCursor {
    fn apply(self, world: &mut World) {
        let mut q_window = world.query_filtered::<&mut Window, With<PrimaryWindow>>();
        let Ok(mut window) = q_window.get_single_mut(world) else {
            return;
        };

        if window.cursor.icon != self.cursor {
            window.cursor.icon = self.cursor;
        }
    }
}

pub trait SetCursorExt<'w, 's, 'a> {
    fn set_cursor(&'a mut self, cursor: CursorIcon);
}

impl<'w, 's, 'a> SetCursorExt<'w, 's, 'a> for Commands<'w, 's> {
    fn set_cursor(&'a mut self, cursor: CursorIcon) {
        self.add(SetCursor { cursor });
    }
}
