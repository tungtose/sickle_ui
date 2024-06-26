//! An example using the widget library to test docking zones and zone splits.
use bevy::prelude::*;

use sickle_math::ease::Ease;
use sickle_ui::{
    dev_panels::hierarchy::{HierarchyTreeViewPlugin, UiHierarchyExt},
    flux_interaction::{FluxInteraction, FluxInteractionUpdate},
    theme::prelude::*,
    ui_builder::{UiBuilder, UiBuilderExt, UiContextRoot, UiRoot},
    ui_style::prelude::*,
    widgets::prelude::*,
    SickleUiPlugin,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Sickle UI - Docking Zone Splits".into(),
                resolution: (1280., 720.).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(SickleUiPlugin)
        .init_resource::<IconCache>()
        .add_plugins(HierarchyTreeViewPlugin)
        .add_plugins((ComponentThemePlugin::<ThemeTestBox>::new(),))
        .add_systems(Startup, setup.in_set(UiStartupSet))
        .add_systems(
            Update,
            (
                update_theme_data_on_press,
                update_test_pseudo_state_on_press,
                log_on_dynamic_style_enter_change,
            )
                .after(FluxInteractionUpdate),
        )
        .run();
}

#[derive(Component)]
pub struct UiCamera;

#[derive(SystemSet, Clone, Hash, Debug, Eq, PartialEq)]
pub struct UiStartupSet;

#[derive(Resource, Debug, Default, Reflect)]
#[reflect(Resource)]
struct IconCache(Vec<Handle<Image>>);

const TEST_CONTEXT: &'static str = "ThemeTestBoxContext";

#[derive(Component, Clone)]
pub struct ThemeTestBox {
    content: Entity,
}

impl UiContext for ThemeTestBox {
    fn get(&self, context: &str) -> Result<Entity, String> {
        if context == TEST_CONTEXT {
            Ok(self.content)
        } else {
            Err("ThemeTestBox has no contexts".into())
        }
    }

    fn contexts(&self) -> Vec<&'static str> {
        vec![TEST_CONTEXT]
    }
}

impl DefaultTheme for ThemeTestBox {
    fn default_theme() -> Option<Theme<Self>> {
        ThemeTestBox::base_theme().into()
    }
}

#[derive(Component)]
pub struct ThemeTestBoxToggle;

impl ThemeTestBox {
    fn base_theme() -> Theme<ThemeTestBox> {
        let base_style = PseudoTheme::deferred(None, |base_style, _| {
            base_style
                .border(UiRect::all(Val::Px(2.)))
                .background_color(Color::BLACK)
                .width(Val::Px(100.))
                .height(Val::Px(100.));

            base_style
                .interactive()
                .border_color(InteractiveVals::new(Color::DARK_GRAY).hover(Color::BEIGE));

            base_style
                .switch_target(TEST_CONTEXT)
                .background_color(Color::BLACK);
        });

        let checked_style = PseudoTheme::build(vec![PseudoState::Checked], |checked_style| {
            checked_style.background_color(Color::GRAY);
        });

        let checked_empty_style = PseudoTheme::build(
            vec![PseudoState::Checked, PseudoState::Empty],
            |checked_empty_style| {
                checked_empty_style.background_color(Color::SEA_GREEN);
            },
        );
        let checked_empty_selected_style = PseudoTheme::build(
            vec![
                PseudoState::Checked,
                PseudoState::Empty,
                PseudoState::Selected,
            ],
            |checked_empty_selected_style| {
                checked_empty_selected_style.background_color(Color::RED);
            },
        );

        Theme::new(vec![
            base_style,
            checked_style,
            checked_empty_style,
            checked_empty_selected_style,
        ])
    }

    fn override_theme() -> Theme<ThemeTestBox> {
        let base_style = PseudoTheme::deferred(None, ThemeTestBox::style_builder);
        Theme::new(vec![base_style])
    }

    fn style_builder(builder: &mut StyleBuilder, data: &ThemeData) {
        let pressed = LoopedAnimationConfig::new(
            0.3,
            Ease::InOutExpo,
            0.3,
            0.1,
            AnimationLoop::PingPongContinous,
        );
        let idle = LoopedAnimationConfig::new(
            0.3,
            Ease::InOutExpo,
            0.3,
            0.1,
            AnimationLoop::Times(3, true),
        );
        let color_bundle = AnimatedVals {
            idle: data.colors().background,
            hover: Color::rgb(0.5, 0.5, 1.).into(),
            press: Color::GREEN.into(),
            cancel: Color::RED.into(),
            hover_alt: Color::GOLD.into(),
            idle_alt: Color::rgb(0.5, 0.5, 1.).into(),
            press_alt: Color::rgb(0.5, 1., 0.5).into(),
            enter_from: Color::WHITE.into(),
            ..default()
        };

        let mut style_animation = AnimationSettings::new();
        style_animation
            .enter(1.0, Ease::Linear, 0.)
            .pointer_enter(0.3, Ease::Linear, 0.5)
            .pointer_leave(0.3, Ease::Linear, 0.5)
            .press(0.3, None, None)
            .cancel(0.3, None, None)
            .cancel_reset(0.3, None, 0.3)
            .idle_from(idle)
            .hover(0.3, Ease::InOutExpo, 0.3, 0.1, AnimationLoop::PingPong(3))
            .pressed_from(pressed);

        builder.border_color(Color::ALICE_BLUE);

        builder
            .animated()
            .background_color(color_bundle)
            .copy_from(style_animation);

        builder
            .switch_target(TEST_CONTEXT)
            .interactive()
            .background_color(InteractiveVals::new(Color::DARK_GRAY).hover(Color::BEIGE));
    }
}

fn update_theme_data_on_press(
    q_test_boxes: Query<&Interaction, (With<ThemeTestBoxToggle>, Changed<Interaction>)>,
    mut theme_data: ResMut<ThemeData>,
) {
    for interaction in &q_test_boxes {
        if *interaction == Interaction::Pressed {
            if theme_data.active_scheme.is_dark() {
                theme_data.active_scheme = Scheme::Light(Contrast::Standard);
            } else {
                theme_data.active_scheme = Scheme::Dark(Contrast::Standard);
            }
        }
    }
}

fn update_test_pseudo_state_on_press(
    mut q_test_boxes: Query<
        (Entity, &FluxInteraction, Option<&mut PseudoStates>),
        (With<ThemeTestBox>, Changed<FluxInteraction>),
    >,
    mut commands: Commands,
) {
    for (entity, interaction, pseudo_states) in &mut q_test_boxes {
        if interaction.is_released() {
            if let Some(mut pseudo_states) = pseudo_states {
                match pseudo_states.get().len() {
                    0 => pseudo_states.add(PseudoState::Checked),
                    1 => pseudo_states.add(PseudoState::Empty),
                    2 => match pseudo_states.has(&PseudoState::Empty) {
                        true => {
                            pseudo_states.add(PseudoState::Selected);
                        }
                        false => {
                            commands.entity(entity).remove::<PseudoStates>();
                        }
                    },
                    3 => {
                        pseudo_states.remove(PseudoState::Empty);
                    }
                    _ => (),
                }
            } else {
                let new_state = PseudoStates::new();
                commands.entity(entity).insert(new_state);
            }
        }
    }
}

fn log_on_dynamic_style_enter_change(
    q_test_boxes: Query<
        &DynamicStyleEnterState,
        (With<ThemeTestBox>, Changed<DynamicStyleEnterState>),
    >,
) {
    for state in &q_test_boxes {
        info!("Enter state changed to: {:?}", state.completed());
    }
}

fn setup(
    asset_server: Res<AssetServer>,
    mut icon_cache: ResMut<IconCache>,
    mut commands: Commands,
) {
    // Workaround for disappearing icons when they are despawned and spawned back in during the same frame
    // Should be fixed in Bevy > 0.13
    let icons_to_cache: Vec<&str> = vec![
        "embedded://sickle_ui/icons/checkmark.png",
        "embedded://sickle_ui/icons/chevron_down.png",
        "embedded://sickle_ui/icons/chevron_left.png",
        "embedded://sickle_ui/icons/chevron_right.png",
        "embedded://sickle_ui/icons/chevron_up.png",
        "embedded://sickle_ui/icons/close.png",
        "embedded://sickle_ui/icons/exit_white.png",
        "embedded://sickle_ui/icons/popout_white.png",
        "embedded://sickle_ui/icons/redo_white.png",
        "embedded://sickle_ui/icons/submenu_white.png",
    ];

    for icon in icons_to_cache.iter() {
        icon_cache.0.push(asset_server.load(*icon));
    }

    // The main camera which will render UI
    let main_camera = commands
        .spawn((
            Camera3dBundle {
                camera: Camera {
                    order: 1,
                    clear_color: Color::BLACK.into(),
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(0., 30., 0.))
                    .looking_at(Vec3::ZERO, Vec3::Y),
                ..Default::default()
            },
            UiCamera,
        ))
        .id();

    // Use the UI builder with plain bundles and direct setting of bundle props
    let mut hierarchy_container = Entity::PLACEHOLDER;
    let mut root_entity = Entity::PLACEHOLDER;
    commands.ui_builder(UiRoot).container(
        (
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },
                ..default()
            },
            TargetCamera(main_camera),
        ),
        |container| {
            container
                .style()
                .background_color(Color::rgb(0.7, 0.7, 0.7));

            container
                .sized_zone(
                    SizedZoneConfig {
                        size: 20.,
                        ..default()
                    },
                    |column| {
                        hierarchy_container = column.id();
                    },
                )
                .style()
                .border(UiRect::right(Val::Px(4.)))
                .background_color(Color::rgb(0.15, 0.155, 0.16));

            container.sized_zone(
                SizedZoneConfig {
                    size: 80.,
                    ..default()
                },
                |main_content| {
                    root_entity = main_content.insert(UiContextRoot).style().id();

                    spawn_test_content(main_content);
                },
            );
        },
    );

    commands
        .ui_builder(hierarchy_container)
        .hierarchy_for(root_entity);
}

fn spawn_test_content(container: &mut UiBuilder<'_, Entity>) {
    container.sized_zone(
        SizedZoneConfig {
            size: 10.,
            ..default()
        },
        |sized_zone| {
            sized_zone
                .docking_zone(
                    SizedZoneConfig {
                        size: 60.,
                        ..default()
                    },
                    false,
                    |tab_container| {
                        for i in 0..10 {
                            tab_container.add_tab(format!("Tab {}", i).into(), |panel| {
                                panel.label(LabelConfig {
                                    label: format!("Tab {} content", i).into(),
                                    ..default()
                                });

                                // TODO: Remove test square once theming is done
                                if i > 0 {
                                    return;
                                }

                                panel.row(|row| {
                                    row.style().justify_content(JustifyContent::Center);

                                    let mut id = Entity::PLACEHOLDER;
                                    row.container(NodeBundle::default(), |row_box| {
                                        id = row_box
                                            .spawn(NodeBundle {
                                                style: Style {
                                                    width: Val::Px(50.),
                                                    height: Val::Px(50.),
                                                    ..default()
                                                },
                                                ..default()
                                            })
                                            .id();
                                    })
                                    .insert(ThemeTestBox { content: id });

                                    row.container(
                                        (
                                            NodeBundle::default(),
                                            ThemeTestBox::override_theme(),
                                            DynamicStyleEnterState::default(),
                                        ),
                                        |row_box| {
                                            id = row_box
                                                .spawn(NodeBundle {
                                                    style: Style {
                                                        width: Val::Px(50.),
                                                        height: Val::Px(50.),
                                                        ..default()
                                                    },
                                                    ..default()
                                                })
                                                .id();
                                        },
                                    )
                                    .insert(ThemeTestBox { content: id });

                                    row.spawn((
                                        ButtonBundle {
                                            style: Style {
                                                width: Val::Px(100.),
                                                height: Val::Px(100.),
                                                ..default()
                                            },
                                            background_color: Color::BISQUE.into(),
                                            ..default()
                                        },
                                        ThemeTestBoxToggle,
                                    ));
                                });
                            });
                        }
                    },
                )
                .style()
                .background_color(Color::rgb(0.3, 0.3, 0.3));
            sized_zone.sized_zone(
                SizedZoneConfig {
                    size: 20.,
                    ..default()
                },
                |sized_zone| {
                    sized_zone
                        .style()
                        .background_color(Color::rgb(0.3, 0.3, 0.3));
                },
            );
            sized_zone.sized_zone(
                SizedZoneConfig {
                    size: 20.,
                    ..default()
                },
                |sized_zone| {
                    sized_zone
                        .style()
                        .background_color(Color::rgb(0.3, 0.3, 0.3));
                },
            );
        },
    );
    container.sized_zone(
        SizedZoneConfig {
            size: 80.,
            ..default()
        },
        |sized_zone| {
            sized_zone.sized_zone(
                SizedZoneConfig {
                    size: 10.,
                    ..default()
                },
                |sized_zone| {
                    sized_zone
                        .style()
                        .background_color(Color::rgb(0.5, 0.5, 0.5));
                },
            );
            sized_zone.docking_zone_split(
                SizedZoneConfig {
                    size: 80.,
                    ..default()
                },
                |zone_split| {
                    zone_split
                        .docking_zone(
                            SizedZoneConfig {
                                size: 20.,
                                ..default()
                            },
                            true,
                            |tab_container| {
                                tab_container.add_tab("Despawnable zone 1".into(), |panel| {
                                    panel.label(LabelConfig {
                                        label: "Despawnable zone 1 content".into(),
                                        ..default()
                                    });
                                });
                            },
                        )
                        .style()
                        .background_color(Color::rgb(0.5, 0.5, 0.5));
                    zone_split
                        .docking_zone(
                            SizedZoneConfig {
                                size: 60.,
                                ..default()
                            },
                            false,
                            |tab_container| {
                                tab_container.add_tab("Static docking zone".into(), |panel| {
                                    panel.label(LabelConfig {
                                        label: "Static docking zone".into(),
                                        ..default()
                                    });
                                });
                            },
                        )
                        .style()
                        .background_color(Color::rgb(0.5, 0.5, 0.5));
                    zone_split
                        .docking_zone(
                            SizedZoneConfig {
                                size: 20.,
                                ..default()
                            },
                            true,
                            |tab_container| {
                                tab_container.add_tab("Despawnable zone 2".into(), |panel| {
                                    panel.label(LabelConfig {
                                        label: "Despawnable zone 2 content".into(),
                                        ..default()
                                    });
                                });
                            },
                        )
                        .style()
                        .background_color(Color::rgb(0.5, 0.5, 0.5));
                },
            );
            sized_zone.sized_zone(
                SizedZoneConfig {
                    size: 10.,
                    ..default()
                },
                |sized_zone| {
                    sized_zone
                        .style()
                        .background_color(Color::rgb(0.5, 0.5, 0.5));
                },
            );
        },
    );
    container.sized_zone(
        SizedZoneConfig {
            size: 10.,
            ..default()
        },
        |sized_zone| {
            sized_zone.sized_zone(
                SizedZoneConfig {
                    size: 50.,
                    ..default()
                },
                |sized_zone| {
                    sized_zone
                        .style()
                        .background_color(Color::rgb(0.7, 0.7, 0.7));
                },
            );
            sized_zone.sized_zone(
                SizedZoneConfig {
                    size: 50.,
                    ..default()
                },
                |sized_zone| {
                    sized_zone
                        .style()
                        .background_color(Color::rgb(0.7, 0.7, 0.7));
                },
            );
        },
    );
}
