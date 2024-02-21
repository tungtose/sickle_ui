use bevy::{
    ecs::system::{EntityCommand, EntityCommands},
    prelude::*,
};
use sickle_macros::StyleCommand;

pub struct UiStyle<'a> {
    commands: EntityCommands<'a>,
}

impl<'a> UiStyle<'a> {
    pub fn id(&self) -> Entity {
        self.commands.id()
    }
}

pub trait UiStyleExt<'a> {
    fn style(&'a mut self, entity: Entity) -> UiStyle<'a>;
}

impl<'a> UiStyleExt<'a> for Commands<'_, '_> {
    fn style(&'a mut self, entity: Entity) -> UiStyle<'a> {
        UiStyle {
            commands: self.entity(entity),
        }
    }
}

#[derive(StyleCommand)]
struct SetNodeWidth {
    width: Val,
}

#[derive(StyleCommand)]
struct SetNodeHeight {
    height: Val,
}

#[derive(StyleCommand)]
struct SetNodeTop {
    top: Val,
}

#[derive(StyleCommand)]
struct SetNodeRight {
    right: Val,
}

#[derive(StyleCommand)]
struct SetNodeBottom {
    bottom: Val,
}

#[derive(StyleCommand)]
struct SetNodeLeft {
    left: Val,
}

#[derive(StyleCommand)]
struct SetNodeFlexDirection {
    flex_direction: FlexDirection,
}

#[derive(StyleCommand)]
struct SetNodeDisplay {
    display: Display,
}

#[derive(StyleCommand)]
struct SetNodeAlignSelf {
    align_self: AlignSelf,
}

#[derive(StyleCommand)]
struct SetNodeJustifySelf {
    justify_self: JustifySelf,
}

struct SetImage {
    path: String,
}

impl EntityCommand for SetImage {
    fn apply(self, entity: Entity, world: &mut World) {
        let handle = if self.path == "" {
            Handle::default()
        } else {
            world.resource::<AssetServer>().load(self.path)
        };

        let mut q_ui_image = world.query::<&mut UiImage>();
        let Ok(mut image) = q_ui_image.get_mut(world, entity) else {
            warn!(
                "Failed to set image on entity {:?}: No UiImage component found!",
                entity
            );
            return;
        };

        if image.texture != handle {
            image.texture = handle;
        }
    }
}

pub trait SetImageExt<'a> {
    fn image(&'a mut self, path: impl Into<String>) -> &mut UiStyle<'a>;
}

impl<'a> SetImageExt<'a> for UiStyle<'a> {
    fn image(&'a mut self, path: impl Into<String>) -> &mut UiStyle<'a> {
        self.commands.add(SetImage { path: path.into() });
        self
    }
}

struct SetEntityVisiblity {
    visibility: Visibility,
}

impl EntityCommand for SetEntityVisiblity {
    fn apply(self, entity: Entity, world: &mut World) {
        let mut q_visibility = world.query::<&mut Visibility>();
        let Ok(mut visiblity) = q_visibility.get_mut(world, entity) else {
            warn!(
                "Failed to set visiblity on entity {:?}: No Visibility component found!",
                entity
            );
            return;
        };

        if *visiblity != self.visibility {
            *visiblity = self.visibility;
        }
    }
}

pub trait SetEntityVisiblityExt<'a> {
    fn visibility(&'a mut self, visibility: Visibility) -> &mut UiStyle<'a>;
}

impl<'a> SetEntityVisiblityExt<'a> for UiStyle<'a> {
    fn visibility(&'a mut self, visibility: Visibility) -> &mut UiStyle<'a> {
        self.commands.add(SetEntityVisiblity { visibility });
        self
    }
}

pub trait SetNodeShowHideExt<'a> {
    fn show(&'a mut self) -> &mut UiStyle<'a>;
    fn hide(&'a mut self) -> &mut UiStyle<'a>;
    fn render(&'a mut self, render: bool) -> &mut UiStyle<'a>;
}

impl<'a> SetNodeShowHideExt<'a> for UiStyle<'a> {
    fn show(&'a mut self) -> &mut UiStyle<'a> {
        self.commands
            .add(SetEntityVisiblity {
                visibility: Visibility::Inherited,
            })
            .add(SetNodeDisplay {
                display: Display::Flex,
            });
        self
    }

    fn hide(&'a mut self) -> &mut UiStyle<'a> {
        self.commands
            .add(SetEntityVisiblity {
                visibility: Visibility::Hidden,
            })
            .add(SetNodeDisplay {
                display: Display::None,
            });
        self
    }

    fn render(&'a mut self, render: bool) -> &mut UiStyle<'a> {
        if render {
            self.commands
                .add(SetEntityVisiblity {
                    visibility: Visibility::Inherited,
                })
                .add(SetNodeDisplay {
                    display: Display::Flex,
                });
        } else {
            self.commands
                .add(SetEntityVisiblity {
                    visibility: Visibility::Hidden,
                })
                .add(SetNodeDisplay {
                    display: Display::None,
                });
        }

        self
    }
}

struct SetBorderColor {
    color: Color,
}

impl EntityCommand for SetBorderColor {
    fn apply(self, entity: Entity, world: &mut World) {
        let mut q_border_color = world.query::<&mut BorderColor>();
        let Ok(mut border_color) = q_border_color.get_mut(world, entity) else {
            warn!(
                "Failed to set border color on entity {:?}: No BorderColor component found!",
                entity
            );
            return;
        };

        if border_color.0 != self.color.into() {
            border_color.0 = self.color.into();
        }
    }
}

pub trait SetBorderColorExt<'a> {
    fn border_color(&'a mut self, color: Color) -> &mut UiStyle<'a>;
}

impl<'a> SetBorderColorExt<'a> for UiStyle<'a> {
    fn border_color(&'a mut self, color: Color) -> &mut UiStyle<'a> {
        self.commands.add(SetBorderColor { color });
        self
    }
}

struct SetBackgroundColor {
    color: Color,
}

impl EntityCommand for SetBackgroundColor {
    fn apply(self, entity: Entity, world: &mut World) {
        let mut q_background_color = world.query::<&mut BackgroundColor>();
        let Ok(mut background_color) = q_background_color.get_mut(world, entity) else {
            warn!(
                "Failed to set background color on entity {:?}: No BackgroundColor component found!",
                entity
            );
            return;
        };

        if background_color.0 != self.color.into() {
            background_color.0 = self.color.into();
        }
    }
}

pub trait SetBackgroundColorExt<'a> {
    fn background_color(&'a mut self, color: Color) -> &mut UiStyle<'a>;
}

impl<'a> SetBackgroundColorExt<'a> for UiStyle<'a> {
    fn background_color(&'a mut self, color: Color) -> &mut UiStyle<'a> {
        self.commands.add(SetBackgroundColor { color });
        self
    }
}
