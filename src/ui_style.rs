use bevy::{ecs::system::Command, prelude::*};
use sickle_macros::StyleCommand;

pub struct UiStyle<'w, 's, 'a> {
    commands: &'a mut Commands<'w, 's>,
    entity: Entity,
}

impl<'w, 's, 'a> UiStyle<'w, 's, 'a> {
    pub fn id(&self) -> Entity {
        self.entity
    }
}

pub trait UiStyleExt<'w, 's, 'a> {
    fn style(&'a mut self, entity: Entity) -> UiStyle<'w, 's, 'a>;
}

impl<'w, 's, 'a> UiStyleExt<'w, 's, 'a> for Commands<'w, 's> {
    fn style(&'a mut self, entity: Entity) -> UiStyle<'w, 's, 'a> {
        UiStyle {
            commands: self,
            entity,
        }
    }
}

#[derive(StyleCommand)]
struct SetEntityWidth {
    entity: Entity,
    width: Val,
}

#[derive(StyleCommand)]
struct SetEntityHeight {
    entity: Entity,
    height: Val,
}

#[derive(StyleCommand)]
struct SetEntityFlexDirection {
    entity: Entity,
    flex_direction: FlexDirection,
}

#[derive(StyleCommand)]
struct SetEntityDisplay {
    entity: Entity,
    display: Display,
}

#[derive(StyleCommand)]
struct SetEntityAlignSelf {
    entity: Entity,
    align_self: AlignSelf,
}

#[derive(StyleCommand)]
struct SetEntityJustifySelf {
    entity: Entity,
    justify_self: JustifySelf,
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
    fn image(&'a mut self, path: impl Into<String>) -> &mut UiStyle<'w, 's, 'a>;
}

impl<'w, 's, 'a> SetImageExt<'w, 's, 'a> for UiStyle<'w, 's, 'a> {
    fn image(&'a mut self, path: impl Into<String>) -> &mut UiStyle<'w, 's, 'a> {
        self.commands.add(SetImage {
            entity: self.entity,
            path: path.into(),
        });
        self
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
    fn visibility(&'a mut self, visibility: Visibility) -> &mut UiStyle<'w, 's, 'a>;
}

impl<'w, 's, 'a> SetEntityVisiblityExt<'w, 's, 'a> for UiStyle<'w, 's, 'a> {
    fn visibility(&'a mut self, visibility: Visibility) -> &mut UiStyle<'w, 's, 'a> {
        self.commands.add(SetEntityVisiblity {
            entity: self.entity,
            visibility,
        });
        self
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
    fn border_color(&'a mut self, color: Color) -> &mut UiStyle<'w, 's, 'a>;
}

impl<'w, 's, 'a> SetBorderColorExt<'w, 's, 'a> for UiStyle<'w, 's, 'a> {
    fn border_color(&'a mut self, color: Color) -> &mut UiStyle<'w, 's, 'a> {
        self.commands.add(SetBorderColor {
            entity: self.entity,
            color,
        });
        self
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
    fn background_color(&'a mut self, color: Color) -> &mut UiStyle<'w, 's, 'a>;
}

impl<'w, 's, 'a> SetBackgroundColorExt<'w, 's, 'a> for UiStyle<'w, 's, 'a> {
    fn background_color(&'a mut self, color: Color) -> &mut UiStyle<'w, 's, 'a> {
        self.commands.add(SetBackgroundColor {
            entity: self.entity,
            color,
        });
        self
    }
}
