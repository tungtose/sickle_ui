use bevy::{
    ecs::{
        entity::Entity,
        query::With,
        system::{Command, Commands, EntityCommand, EntityCommands},
        world::World,
    },
    log::warn,
    text::{Text, TextSection, TextStyle},
    window::{CursorIcon, PrimaryWindow, Window},
};

struct SetTextSections {
    sections: Vec<TextSection>,
}

impl EntityCommand for SetTextSections {
    fn apply(self, entity: Entity, world: &mut World) {
        let mut q_text = world.query::<&mut Text>();
        let Ok(mut text) = q_text.get_mut(world, entity) else {
            warn!(
                "Failed to set text sections on entity {:?}: No Text component found!",
                entity
            );
            return;
        };

        text.sections = self.sections;
    }
}

pub trait SetTextSectionsExt<'w, 's, 'a> {
    fn set_text_sections(
        &'a mut self,
        sections: Vec<TextSection>,
    ) -> &mut EntityCommands<'w, 's, 'a>;
}

impl<'w, 's, 'a> SetTextSectionsExt<'w, 's, 'a> for EntityCommands<'w, 's, 'a> {
    fn set_text_sections(
        &'a mut self,
        sections: Vec<TextSection>,
    ) -> &mut EntityCommands<'w, 's, 'a> {
        self.add(SetTextSections { sections });
        self
    }
}

struct SetText {
    text: String,
    style: TextStyle,
}

impl EntityCommand for SetText {
    fn apply(self, entity: Entity, world: &mut World) {
        let mut q_text = world.query::<&mut Text>();
        let Ok(mut text) = q_text.get_mut(world, entity) else {
            warn!(
                "Failed to set text on entity {:?}: No Text component found!",
                entity
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
    ) -> &mut EntityCommands<'w, 's, 'a>;
}

impl<'w, 's, 'a> SetTextExt<'w, 's, 'a> for EntityCommands<'w, 's, 'a> {
    fn set_text(
        &'a mut self,
        text: impl Into<String>,
        style: Option<TextStyle>,
    ) -> &mut EntityCommands<'w, 's, 'a> {
        self.add(SetText {
            text: text.into(),
            style: style.unwrap_or_default(),
        });

        self
    }
}

// TODO: Move to style and apply to Node's window
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
