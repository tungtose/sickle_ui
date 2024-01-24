use bevy::{ecs::system::EntityCommands, prelude::*};
use sickle_math::ease::Ease;

use crate::{
    animated_interaction::{AnimatedInteraction, AnimationConfig},
    interactions::InteractiveBackground,
    ui_builder::*,
    FluxInteraction, FluxInteractionUpdate, TrackedInteraction,
};

use super::prelude::{LabelConfig, UiContainerExt, UiLabelExt};

const MENU_CONTAINER_Z_INDEX: i32 = 100000;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                handle_click_or_touch.after(FluxInteractionUpdate),
                update_menu_container_visibility,
            )
                .chain(),
        );
    }
}

fn handle_click_or_touch(
    r_mouse: Res<Input<MouseButton>>,
    r_touches: Res<Touches>,
    mut q_menus: Query<(Entity, &mut Menu, Ref<FluxInteraction>)>,
) {
    if r_mouse.any_just_pressed([MouseButton::Left, MouseButton::Middle, MouseButton::Right])
        || r_touches.any_just_pressed()
    {
        let any_pressed = q_menus
            .iter()
            .any(|(_, _, f)| *f == FluxInteraction::Pressed);
        if !any_pressed {
            for (_, mut menu, _) in &mut q_menus {
                menu.is_open = false;
            }
            return;
        }
    }

    let any_changed = q_menus.iter().any(|(_, _, f)| f.is_changed());
    if !any_changed {
        return;
    }

    let any_open = q_menus.iter().any(|(_, m, _)| m.is_open);
    let mut open: Option<Entity> =
        if let Some((entity, _, _)) = q_menus.iter().find(|(_, m, _)| m.is_open) {
            entity.into()
        } else {
            None
        };

    for (entity, menu, interaction) in &mut q_menus {
        if interaction.is_changed() {
            if (menu.is_open && *interaction == FluxInteraction::Pressed)
                || (!menu.is_open && *interaction == FluxInteraction::Released)
            {
                open = None;
                break;
            }
            if *interaction == FluxInteraction::Pressed || *interaction == FluxInteraction::Released
            {
                open = entity.into();
                break;
            } else if any_open && *interaction == FluxInteraction::PointerEnter {
                open = entity.into();
                break;
            }
        }
    }

    for (entity, mut menu, _) in &mut q_menus {
        if let Some(open_dropdown) = open {
            if entity == open_dropdown {
                if !menu.is_open {
                    menu.is_open = true;
                }
            } else if menu.is_open {
                menu.is_open = false;
            }
        } else if menu.is_open {
            menu.is_open = false;
        }
    }
}

fn update_menu_container_visibility(
    mut q_menus: Query<(Ref<Menu>, &mut BorderColor)>,
    mut q_style: Query<&mut Style>,
) {
    for (menu, mut border_color) in &mut q_menus {
        if !menu.is_changed() {
            continue;
        }

        let Ok(mut container_style) = q_style.get_mut(menu.container) else {
            continue;
        };

        if menu.is_open {
            container_style.display = Display::Flex;
            border_color.0 = Color::WHITE.into();
        } else if container_style.display != Display::None {
            container_style.display = Display::None;
            border_color.0 = Color::NONE.into();
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct Menu {
    container: Entity,
    is_open: bool,
}

impl Default for Menu {
    fn default() -> Self {
        Self {
            container: Entity::PLACEHOLDER,
            is_open: false,
        }
    }
}

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct MenuItem;

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct MenuItemSeparator;

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct MenuConfig {
    pub name: String,
    pub alt_code: Option<KeyCode>,
    //pub icon: Option<Handle<Image>>,
}

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct MenuItemConfig {
    pub name: String,
    pub icon: Option<Handle<Image>>,
    pub alt_code: Option<KeyCode>,
    pub shortcut: Option<Vec<KeyCode>>,
}

impl Menu {
    fn base_tween() -> AnimationConfig {
        AnimationConfig {
            duration: 0.1,
            easing: Ease::OutExpo,
            ..default()
        }
    }

    fn button() -> impl Bundle {
        (
            ButtonBundle {
                style: Style {
                    padding: UiRect::axes(Val::Px(10.), Val::Px(5.)),
                    border: UiRect::horizontal(Val::Px(1.)),
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: Color::NONE.into(),
                border_color: Color::NONE.into(),
                ..default()
            },
            TrackedInteraction::default(),
            InteractiveBackground {
                highlight: Some(Color::rgba(9., 8., 7., 0.5)),
                ..default()
            },
            AnimatedInteraction::<InteractiveBackground> {
                tween: Menu::base_tween(),
                ..default()
            },
        )
    }

    fn container() -> impl Bundle {
        NodeBundle {
            style: Style {
                top: Val::Px(22.),
                left: Val::Px(-1.),
                position_type: PositionType::Absolute,
                border: UiRect::px(1., 1., 0., 1.),
                padding: UiRect::px(5., 5., 5., 10.),
                flex_direction: FlexDirection::Column,
                align_self: AlignSelf::End,
                align_items: AlignItems::Stretch,
                display: Display::None,
                ..default()
            },
            z_index: ZIndex::Global(MENU_CONTAINER_Z_INDEX),
            background_color: Color::rgb(0.7, 0.6, 0.5).into(),
            border_color: Color::WHITE.into(),
            ..default()
        }
    }
}

impl MenuItem {
    fn base_tween() -> AnimationConfig {
        AnimationConfig {
            duration: 0.1,
            easing: Ease::OutExpo,
            ..default()
        }
    }

    fn button() -> impl Bundle {
        (
            ButtonBundle {
                style: Style {
                    padding: UiRect::all(Val::Px(5.)),
                    justify_content: JustifyContent::End,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: Color::NONE.into(),
                ..default()
            },
            TrackedInteraction::default(),
            InteractiveBackground {
                highlight: Some(Color::rgba(9., 8., 7., 0.5)),
                ..default()
            },
            AnimatedInteraction::<InteractiveBackground> {
                tween: MenuItem::base_tween(),
                ..default()
            },
        )
    }

    fn shortcut() -> impl Bundle {
        NodeBundle {
            style: Style {
                margin: UiRect::left(Val::Px(20.)),
                justify_content: JustifyContent::End,
                flex_wrap: FlexWrap::NoWrap,
                flex_grow: 2.,
                ..default()
            },
            ..default()
        }
    }

    fn icon_spacer() -> impl Bundle {
        NodeBundle {
            style: Style {
                width: Val::Px(12.),
                ..default()
            },
            ..default()
        }
    }

    fn icon(texture: Handle<Image>) -> impl Bundle {
        ImageBundle {
            style: Style {
                width: Val::Px(12.),
                ..default()
            },
            image: UiImage::new(texture),
            ..default()
        }
    }

    fn shortcut_text(keycodes: Vec<KeyCode>) -> String {
        keycodes
            .iter()
            .map(MenuItem::keycode_text)
            .collect::<Vec<String>>()
            .join("+")
    }

    fn keycode_text(keycode: &KeyCode) -> String {
        let formatted = format!("{:?}", keycode);
        let formatted_str = formatted.as_str();

        let renamed = match keycode {
            KeyCode::Key1 => "1",
            KeyCode::Key2 => "2",
            KeyCode::Key3 => "3",
            KeyCode::Key4 => "4",
            KeyCode::Key5 => "5",
            KeyCode::Key6 => "6",
            KeyCode::Key7 => "7",
            KeyCode::Key8 => "8",
            KeyCode::Key9 => "9",
            KeyCode::Key0 => "0",
            KeyCode::Escape => "ESC",
            KeyCode::Insert => "Ins",
            KeyCode::Delete => "Del",
            KeyCode::Apostrophe => "'",
            KeyCode::Asterisk => "*",
            KeyCode::Plus => "+",
            KeyCode::At => "@",
            KeyCode::Backslash => "\\",
            KeyCode::Colon => ":",
            KeyCode::Comma => ",",
            KeyCode::NumpadDecimal => ".",
            KeyCode::NumpadDivide => "/",
            KeyCode::Equals => "=",
            KeyCode::Grave => "`",
            KeyCode::AltLeft => "Alt",
            KeyCode::BracketLeft => "[",
            KeyCode::ControlLeft => "Ctrl",
            KeyCode::ShiftLeft => "Shift",
            KeyCode::Minus => "-",
            KeyCode::NumpadMultiply => "*",
            KeyCode::NumpadComma => ",",
            KeyCode::NumpadEquals => "=",
            KeyCode::Period => ",",
            KeyCode::AltRight => "Alt",
            KeyCode::BracketRight => "]",
            KeyCode::ControlRight => "Ctrl",
            KeyCode::ShiftRight => "Shift",
            KeyCode::Semicolon => ";",
            KeyCode::Slash => "/",
            KeyCode::NumpadSubtract => "-",
            KeyCode::Underline => "_",
            _ => formatted_str,
        };

        renamed.to_string()
    }
}

impl MenuItemSeparator {
    fn separator() -> impl Bundle {
        NodeBundle {
            style: Style {
                min_width: Val::Px(100.),
                height: Val::Px(1.),
                margin: UiRect::px(5., 5., 5., 5.),
                ..default()
            },
            background_color: Color::GRAY.into(),
            ..default()
        }
    }
}

pub trait UiMenuExt<'w, 's> {
    fn menu<'a>(
        &'a mut self,
        config: MenuConfig,
        spawn_items: impl FnOnce(&mut UiBuilder),
    ) -> EntityCommands<'w, 's, 'a>;
}

impl<'w, 's> UiMenuExt<'w, 's> for UiBuilder<'w, 's, '_> {
    fn menu<'a>(
        &'a mut self,
        config: MenuConfig,
        spawn_items: impl FnOnce(&mut UiBuilder),
    ) -> EntityCommands<'w, 's, 'a> {
        let mut container = Entity::PLACEHOLDER;
        let mut menu = self.container(Menu::button(), |menu_button| {
            container = menu_button.container(Menu::container(), spawn_items).id();
            menu_button.label(LabelConfig {
                label: config.name.clone(),
                ..default()
            });
        });

        menu.insert((
            Menu {
                container,
                ..default()
            },
            config,
        ));

        menu
    }
}

pub trait UiMenuItemExt<'w, 's> {
    fn menu_item<'a>(&'a mut self, config: MenuItemConfig) -> EntityCommands<'w, 's, 'a>;
}

impl<'w, 's> UiMenuItemExt<'w, 's> for UiBuilder<'w, 's, '_> {
    fn menu_item<'a>(&'a mut self, config: MenuItemConfig) -> EntityCommands<'w, 's, 'a> {
        let name = config.name.clone();
        let shortcut_text: Option<String> = if let Some(keycodes) = config.shortcut.clone() {
            Some(MenuItem::shortcut_text(keycodes))
        } else {
            None
        };
        let icon = config.icon.clone();

        self.container((MenuItem::button(), config), |button| {
            if let Some(icon) = icon {
                button.spawn(MenuItem::icon(icon));
            } else {
                button.spawn(MenuItem::icon_spacer());
            }

            button.label(LabelConfig {
                label: name,
                margin: UiRect::horizontal(Val::Px(5.)),
                ..default()
            });

            if let Some(shortcut_text) = shortcut_text {
                button.container(MenuItem::shortcut(), |shortcut| {
                    shortcut.label(LabelConfig {
                        label: shortcut_text,
                        margin: UiRect::horizontal(Val::Px(5.)),
                        ..default()
                    });
                });
            } else {
                button.spawn(MenuItem::shortcut());
            }

            button.spawn(MenuItem::icon_spacer());
        })
    }
}

pub trait UiMenuItemSeparatorExt<'w, 's> {
    fn menu_item_separator<'a>(&'a mut self) -> EntityCommands<'w, 's, 'a>;
}

impl<'w, 's> UiMenuItemSeparatorExt<'w, 's> for UiBuilder<'w, 's, '_> {
    fn menu_item_separator<'a>(&'a mut self) -> EntityCommands<'w, 's, 'a> {
        self.spawn(MenuItemSeparator::separator())
    }
}
