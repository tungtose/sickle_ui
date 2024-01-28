use bevy::{
    ecs::system::{Command, EntityCommands},
    prelude::*,
    ui::FocusPolicy,
};

use crate::ui_builder::*;

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct LabelConfig {
    pub label: String,
    pub color: Color,
    pub margin: UiRect,
    pub wrap: FlexWrap,
    pub flex_grow: f32,
}

impl Default for LabelConfig {
    fn default() -> Self {
        Self {
            label: "Label".into(),
            color: Color::BLACK,
            margin: Default::default(),
            wrap: FlexWrap::NoWrap,
            flex_grow: 0.,
        }
    }
}

impl LabelConfig {
    pub fn from(label: impl Into<String>) -> LabelConfig {
        LabelConfig {
            label: label.into(),
            ..default()
        }
    }

    fn text_style(&self) -> TextStyle {
        TextStyle {
            color: self.color,
            font_size: 16.,
            ..default()
        }
    }

    fn frame(self) -> impl Bundle {
        let mut section = Text::from_section(self.label.clone(), self.text_style());

        if self.wrap == FlexWrap::NoWrap {
            section = section.with_no_wrap();
        }

        (
            TextBundle {
                style: Style {
                    align_self: AlignSelf::Center,
                    margin: self.margin,
                    flex_wrap: self.wrap,
                    flex_grow: self.flex_grow,
                    ..default()
                },
                text: section,
                focus_policy: FocusPolicy::Pass,
                ..default()
            },
            self,
        )
    }
}

pub trait UiLabelExt<'w, 's> {
    fn label<'a>(&'a mut self, config: LabelConfig) -> EntityCommands<'w, 's, 'a>;
}

impl<'w, 's> UiLabelExt<'w, 's> for UiBuilder<'w, 's, '_> {
    fn label<'a>(&'a mut self, config: LabelConfig) -> EntityCommands<'w, 's, 'a> {
        self.spawn((config.frame(), Label))
    }
}

struct UpdateLabelText {
    entity: Entity,
    text: String,
}

impl Command for UpdateLabelText {
    fn apply(self, world: &mut World) {
        let mut q_text = world.query::<&mut Text>();
        let mut q_config = world.query::<&LabelConfig>();
        let Ok(config) = q_config.get(world, self.entity) else {
            return;
        };
        let style = config.text_style();
        let Ok(mut text) = q_text.get_mut(world, self.entity) else {
            return;
        };

        text.sections = vec![TextSection::new(self.text, style)];
    }
}

pub trait SetLabelTextExt<'w, 's, 'a> {
    fn set_label_text(&'a mut self, text: impl Into<String>) -> EntityCommands<'w, 's, 'a>;
}

impl<'w, 's, 'a> SetLabelTextExt<'w, 's, 'a> for EntityCommands<'w, 's, 'a> {
    fn set_label_text(&'a mut self, text: impl Into<String>) -> EntityCommands<'w, 's, 'a> {
        let entity = self.id();
        self.commands().add(UpdateLabelText {
            entity,
            text: text.into(),
        });

        self.commands().entity(entity)
    }
}
