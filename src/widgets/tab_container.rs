use bevy::prelude::*;

use crate::{
    ui_builder::{UiBuilder, UiBuilderExt},
    ui_style::{SetBackgroundColorExt, SetNodeShowHideExt, UiStyleExt},
    TrackedInteraction,
};

use super::{
    panel::Panel,
    prelude::{LabelConfig, UiContainerExt, UiLabelExt, UiScrollViewExt},
};

pub struct TabContainerPlugin;

impl Plugin for TabContainerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (process_new_panels, process_removed_panels).chain(),
        )
        .add_systems(
            Update,
            (
                process_new_tab_containers,
                update_tab_container_on_press,
                update_tab_container_on_change,
            )
                .chain(),
        );
    }
}

fn process_new_tab_containers(
    q_added_containers: Query<(Entity, &TabContainer, &Children), Added<TabContainer>>,
    q_panels: Query<(Entity, &Panel)>,
    mut commands: Commands,
) {
    for (entity, container, children) in &q_added_containers {
        for child in children {
            let Ok((panel_entity, panel)) = q_panels.get(*child) else {
                continue;
            };

            commands.entity(panel_entity).set_parent(container.panel);
            commands
                .ui_builder(container.bar.into())
                .tab(panel.title(), entity, panel_entity);
            commands.style(panel_entity).hide();
        }
    }
}

fn process_new_panels() {}
fn process_removed_panels() {}

fn update_tab_container_on_press(
    q_tabs: Query<(Entity, &Tab, &Interaction), Changed<Interaction>>,
    q_tab: Query<Entity, With<Tab>>,
    q_children: Query<&Children>,
    mut q_tab_container: Query<&mut TabContainer>,
) {
    for (tab_entity, tab, interaction) in &q_tabs {
        if *interaction == Interaction::Pressed {
            let Ok(mut tab_container) = q_tab_container.get_mut(tab.container) else {
                continue;
            };

            let Ok(tabs) = q_children.get(tab_container.bar) else {
                continue;
            };

            for (i, id) in tabs.iter().enumerate() {
                if let Ok(_) = q_tab.get(*id) {
                    if *id == tab_entity {
                        tab_container.active = i;
                    }
                }
            }
        }
    }
}

fn update_tab_container_on_change(
    q_tab_containers: Query<&TabContainer, Changed<TabContainer>>,
    q_tab: Query<Entity, With<Tab>>,
    q_children: Query<&Children>,
    q_panel: Query<Entity, With<Panel>>,
    mut commands: Commands,
) {
    for tab_container in &q_tab_containers {
        info!("Tab container changed: {:?}", tab_container);
        let Ok(tabs) = q_children.get(tab_container.bar) else {
            info!("Tab bar has no tabs: {:?}", tab_container.bar);
            continue;
        };

        for (i, id) in tabs.iter().enumerate() {
            if let Ok(tab) = q_tab.get(*id) {
                if i == tab_container.active {
                    commands.style(tab).background_color(Color::GRAY);
                } else {
                    commands.style(tab).background_color(Color::NONE);
                }
            }
        }

        let Ok(panels) = q_children.get(tab_container.panel) else {
            continue;
        };

        for (i, id) in panels.iter().enumerate() {
            if let Ok(panel) = q_panel.get(*id) {
                if i == tab_container.active {
                    commands.style(panel).show();
                } else {
                    commands.style(panel).hide();
                }
            }
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct Tab {
    container: Entity,
    panel: Entity,
}

impl Default for Tab {
    fn default() -> Self {
        Self {
            container: Entity::PLACEHOLDER,
            panel: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct TabBar {
    container: Entity,
}

impl Default for TabBar {
    fn default() -> Self {
        Self {
            container: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct TabPanel {
    container: Entity,
}

impl Default for TabPanel {
    fn default() -> Self {
        Self {
            container: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct TabContainer {
    active: usize,
    bar: Entity,
    panel: Entity,
}

impl Default for TabContainer {
    fn default() -> Self {
        Self {
            active: 0,
            bar: Entity::PLACEHOLDER,
            panel: Entity::PLACEHOLDER,
        }
    }
}

impl TabContainer {
    fn frame() -> impl Bundle {
        (
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ..default()
            },
            Interaction::default(),
        )
    }

    fn bar() -> impl Bundle {
        (
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Px(30.),
                    border: UiRect::bottom(Val::Px(1.)),
                    ..default()
                },
                border_color: Color::DARK_GRAY.into(),
                ..default()
            },
            Interaction::default(),
        )
    }

    fn panel() -> impl Bundle {
        NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ..default()
        }
    }

    fn tab() -> impl Bundle {
        (
            NodeBundle {
                style: Style {
                    padding: UiRect::axes(Val::Px(10.), Val::Px(5.)),
                    border: UiRect::horizontal(Val::Px(1.)),
                    ..default()
                },
                border_color: Color::DARK_GRAY.into(),
                ..default()
            },
            Interaction::default(),
            TrackedInteraction::default(),
        )
    }
}

pub trait UiTabContainerExt<'w, 's> {
    fn tab_container<'a>(
        &'a mut self,
        spawn_children: impl FnOnce(&mut UiBuilder),
    ) -> UiBuilder<'w, 's, 'a>;
}

impl<'w, 's> UiTabContainerExt<'w, 's> for UiBuilder<'w, 's, '_> {
    fn tab_container<'a>(
        &'a mut self,
        spawn_children: impl FnOnce(&mut UiBuilder),
    ) -> UiBuilder<'w, 's, 'a> {
        let mut bar = Entity::PLACEHOLDER;
        let mut panel = Entity::PLACEHOLDER;

        let mut container = self.container(TabContainer::frame(), |container| {
            let container_id = container.id();

            bar = container
                .spawn((
                    TabContainer::bar(),
                    TabBar {
                        container: container_id,
                    },
                ))
                .id();

            container
                .scroll_view(None, |scroll_view| {
                    panel = scroll_view.id();
                })
                .insert((
                    TabContainer::panel(),
                    TabPanel {
                        container: container_id,
                    },
                ));

            spawn_children(container);
        });

        container.insert(TabContainer {
            bar,
            panel,
            ..default()
        });

        container
    }
}

trait UiTabExt<'w, 's> {
    fn tab<'a>(
        &'a mut self,
        title: String,
        tab_container: Entity,
        panel: Entity,
    ) -> UiBuilder<'w, 's, 'a>;
}

impl<'w, 's> UiTabExt<'w, 's> for UiBuilder<'w, 's, '_> {
    fn tab<'a>(
        &'a mut self,
        title: String,
        tab_container: Entity,
        panel: Entity,
    ) -> UiBuilder<'w, 's, 'a> {
        self.container(
            (
                TabContainer::tab(),
                Tab {
                    container: tab_container,
                    panel,
                },
            ),
            |container| {
                container.label(LabelConfig {
                    label: title,
                    ..default()
                });
            },
        )
    }
}
