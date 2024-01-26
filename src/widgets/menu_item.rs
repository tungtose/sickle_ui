use bevy::{ecs::system::EntityCommands, prelude::*};
use sickle_math::ease::Ease;

use crate::{
    animated_interaction::{AnimatedInteraction, AnimationConfig},
    input_extension::{ShortcutTextExt, SymmetricKeysExt},
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
                    update_menu_item_on_config_change,
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
                        .all(|keycode| r_keys.symmetry_pressed(keycode))
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

fn update_menu_item_on_config_change(
    q_menu_items: Query<(Entity, &MenuItemConfig), Changed<MenuItemConfig>>,
    mut commands: Commands,
) {
    for (entity, config) in &q_menu_items {
        let mut entity_commands = commands.entity(entity);
        let mut button = entity_commands.despawn_descendants().ui_builder();

        let name = config.name.clone();
        let shortcut_text: Option<String> = match &config.shortcut {
            Some(vec) => vec.shortcut_text().into(),
            None => None,
        };
        let leading = config.leading_icon.clone();
        let trailing = config.trailing_icon.clone();

        if let Some(leading) = leading {
            button.spawn(MenuItem::icon(leading));
        } else {
            button.spawn(MenuItem::icon_spacer());
        }

        if let Some(shortcut_text) = shortcut_text {
            button.label(LabelConfig {
                label: name,
                margin: UiRect::horizontal(Val::Px(5.)),
                ..default()
            });
            button.container(MenuItem::shortcut(), |shortcut| {
                shortcut.label(LabelConfig {
                    label: shortcut_text,
                    margin: UiRect::horizontal(Val::Px(5.)),
                    ..default()
                });
            });
        } else {
            button.label(LabelConfig {
                label: name,
                margin: UiRect::horizontal(Val::Px(5.)),
                flex_grow: 1.,
                ..default()
            });
        }

        if let Some(trailing) = trailing {
            button.spawn(MenuItem::icon(trailing));
        } else {
            button.spawn(MenuItem::icon_spacer());
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
    pub leading_icon: Option<Handle<Image>>,
    pub trailing_icon: Option<Handle<Image>>,
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
                highlight: Color::rgba(9., 8., 7., 0.5).into(),
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
}

pub trait UiMenuItemExt<'w, 's> {
    fn menu_item<'a>(&'a mut self, config: MenuItemConfig) -> EntityCommands<'w, 's, 'a>;
}

impl<'w, 's> UiMenuItemExt<'w, 's> for UiBuilder<'w, 's, '_> {
    fn menu_item<'a>(&'a mut self, config: MenuItemConfig) -> EntityCommands<'w, 's, 'a> {
        self.spawn((MenuItem::button(), config))
    }
}