use bevy::{ecs::system::EntityCommands, prelude::*};
use sickle_math::ease::Ease;

use crate::{
    animated_interaction::{AnimatedInteraction, AnimationConfig},
    interactions::InteractiveBackground,
    ui_builder::*,
    FluxInteraction, FluxInteractionUpdate, TrackedInteraction,
};

use super::prelude::{LabelConfig, UiContainerExt, UiLabelExt};

pub struct MenuItemPlugin;

impl Plugin for MenuItemPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Update, MenuItemUpdate.after(FluxInteractionUpdate))
            .add_systems(
                Update,
                (
                    update_menu_item_on_change,
                    update_menu_item_on_pressed,
                    update_menu_item_on_key_press,
                )
                    .chain()
                    .in_set(MenuItemUpdate),
            );
    }
}

#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct MenuItemUpdate;

fn update_menu_item_on_pressed(
    mut q_menu_items: Query<(&mut MenuItem, &FluxInteraction), Changed<FluxInteraction>>,
) {
    for (mut item, interaction) in &mut q_menu_items {
        if *interaction == FluxInteraction::Released {
            item.interacted = true;
        }
    }
}

fn update_menu_item_on_key_press(
    mut q_menu_items: Query<(&mut MenuItem, &MenuItemConfig)>,
    r_keys: Res<Input<KeyCode>>,
) {
    if !r_keys.is_changed() {
        return;
    }

    for (mut item, config) in &mut q_menu_items {
        if let Some(shortcut) = &config.shortcut {
            if shortcut.len() == 0 {
                continue;
            }

            let main_key = shortcut.last().unwrap().clone();
            if r_keys.just_pressed(main_key) {
                if shortcut.len() > 1 {
                    if shortcut
                        .iter()
                        .take(shortcut.len() - 1)
                        .map(|c| c.clone())
                        .all(|keycode| match keycode {
                            KeyCode::AltLeft | KeyCode::AltRight => {
                                r_keys.any_pressed([KeyCode::AltLeft, KeyCode::AltRight])
                            }
                            KeyCode::ControlLeft | KeyCode::ControlRight => {
                                r_keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
                            }
                            KeyCode::ShiftLeft | KeyCode::ShiftRight => {
                                r_keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight])
                            }
                            KeyCode::SuperLeft | KeyCode::SuperRight => {
                                r_keys.any_pressed([KeyCode::SuperLeft, KeyCode::SuperRight])
                            }
                            _ => r_keys.pressed(keycode),
                        })
                    {
                        item.interacted = true;
                    }
                } else {
                    item.interacted = true;
                }
            }
        }
    }
}

fn update_menu_item_on_change(mut q_menu_items: Query<&mut MenuItem, Changed<MenuItem>>) {
    for mut item in &mut q_menu_items {
        if item.interacted {
            item.interacted = false;
        }
    }
}

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct MenuItem {
    interacted: bool,
}

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct MenuItemConfig {
    pub name: String,
    pub icon: Option<Handle<Image>>,
    pub alt_code: Option<KeyCode>,
    pub shortcut: Option<Vec<KeyCode>>,
}

impl MenuItem {
    pub fn interacted(&self) -> bool {
        self.interacted
    }

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
            MenuItem { interacted: false },
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
