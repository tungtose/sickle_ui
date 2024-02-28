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
            Update,
            (
                process_tab_container_content_change,
                process_tab_viewport_content_change,
                update_tab_container_on_press,
                constrain_tab_container_active_tab,
                update_tab_container_on_change,
            )
                .chain(),
        );
    }
}

fn process_tab_container_content_change(
    q_tab_containers: Query<(&TabContainer, &Children), Changed<Children>>,
    q_panel: Query<Entity, With<Panel>>,
    mut commands: Commands,
) {
    for (container, children) in &q_tab_containers {
        for child in children {
            let Ok(panel_entity) = q_panel.get(*child) else {
                continue;
            };

            commands.entity(panel_entity).set_parent(container.viewport);
        }
    }
}

fn process_tab_viewport_content_change(
    q_tab_viewports: Query<(Entity, &TabViewport, &Children), Changed<Children>>,
    q_tab_container: Query<(Entity, &TabContainer)>,
    q_children: Query<&Children>,
    q_tab: Query<&Tab>,
    q_panel: Query<&Panel>,
    mut commands: Commands,
) {
    for (entity, viewport, children) in &q_tab_viewports {
        let Ok((tab_container_id, tab_container)) = q_tab_container.get(viewport.container) else {
            error!("Missing tab container for viewport {:?}", entity);
            continue;
        };

        let tab_to_panel_ids: Vec<(Entity, Entity)> =
            if let Ok(tab_bar_children) = q_children.get(tab_container.bar) {
                tab_bar_children
                    .iter()
                    .filter(|child| {
                        if let Ok(_) = q_tab.get(**child) {
                            return true;
                        }
                        false
                    })
                    .map(|child| (*child, q_tab.get(*child).unwrap().panel))
                    .collect()
            } else {
                Vec::new()
            };

        let panels: Vec<(Entity, &Panel)> = children
            .iter()
            .filter(|child| {
                if let Ok(_) = q_panel.get(**child) {
                    return true;
                }
                false
            })
            .map(|child| (*child, q_panel.get(*child).unwrap()))
            .collect();

        for (panel_id, panel) in &panels {
            if !tab_to_panel_ids.iter().any(|(_, p_id)| *p_id == *panel_id) {
                commands.ui_builder(tab_container.bar.into()).tab(
                    panel.title(),
                    tab_container_id,
                    *panel_id,
                );
            }
        }

        for (tab_id, panel_id) in tab_to_panel_ids {
            if !panels.iter().any(|(p_id, _)| *p_id == panel_id) {
                commands.entity(tab_id).despawn_recursive();
            }
        }
    }
}

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

fn constrain_tab_container_active_tab(
    q_changed_tab_bars: Query<(Entity, &TabBar, &Children), Changed<Children>>,
    q_tab: Query<&Tab>,
    mut q_tab_container: Query<&mut TabContainer>,
) {
    for (entity, tab_bar, children) in &q_changed_tab_bars {
        let Ok(mut container) = q_tab_container.get_mut(tab_bar.container) else {
            error!("Missing tab container of tab bar {:?}", entity);
            continue;
        };

        let tab_count = children
            .iter()
            .filter(|child| {
                if let Ok(_) = q_tab.get(**child) {
                    return true;
                }
                false
            })
            .count();

        if container.tab_count != tab_count {
            container.tab_count = tab_count;
        }

        if container.active >= tab_count {
            container.active = tab_count - 1;
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
        let Ok(tabs) = q_children.get(tab_container.bar) else {
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

        let Ok(panels) = q_children.get(tab_container.viewport) else {
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
pub struct TabViewport {
    container: Entity,
}

impl Default for TabViewport {
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
    viewport: Entity,
    tab_count: usize,
}

impl Default for TabContainer {
    fn default() -> Self {
        Self {
            active: 0,
            tab_count: 0,
            bar: Entity::PLACEHOLDER,
            viewport: Entity::PLACEHOLDER,
        }
    }
}

impl TabContainer {
    pub fn bar_id(&self) -> Entity {
        self.bar
    }

    pub fn tab_count(&self) -> usize {
        self.tab_count
    }

    pub fn set_active(&mut self, active: usize) {
        self.active = active;
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
        let mut viewport = Entity::PLACEHOLDER;

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

            container.scroll_view(None, |scroll_view| {
                viewport = scroll_view
                    .insert(TabViewport {
                        container: container_id,
                    })
                    .id();
            });

            spawn_children(container);
        });

        container.insert(TabContainer {
            bar,
            viewport,
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
