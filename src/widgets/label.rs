use bevy::{ecs::system::EntityCommands, prelude::*, ui::FocusPolicy};

use crate::ui_builder::*;

#[derive(Debug)]
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
            label: String::from("Label"),
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

    fn frame(&self) -> impl Bundle {
        let mut section = Text::from_section(
            self.label.clone(),
            TextStyle {
                color: self.color,
                ..default()
            },
        );

        if self.wrap == FlexWrap::NoWrap {
            section = section.with_no_wrap();
        }

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
        }
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
