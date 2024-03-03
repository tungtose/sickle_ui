use bevy::{
    ecs::system::{Command, CommandQueue},
    prelude::*,
};
use sickle_math::ease::Ease;

use crate::{
    animated_interaction::{AnimatedInteraction, AnimationConfig},
    drag_interaction::{DragState, Draggable, DraggableUpdate},
    interactions::InteractiveBackground,
    ui_builder::{UiBuilder, UiBuilderExt},
    ui_style::{
        SetBackgroundColorExt, SetFluxInteractionExt, SetNodeLeftExt, SetNodePositionTypeExt,
        SetNodeShowHideExt, SetZIndexExt, UiStyleExt,
    },
    FluxInteraction, PrevInteraction, TrackedInteraction,
};

use super::{
    context_menu::ContextMenuUpdate,
    floating_panel::{FloatingPanel, FloatingPanelUpdate},
    panel::Panel,
    prelude::{
        ContextMenuGenerator, FloatingPanelConfig, FloatingPanelLayout, GenerateContextMenu,
        LabelConfig, MenuItem, MenuItemConfig, MenuItemUpdate, ReflectContextMenuGenerator,
        UiContainerExt, UiFloatingPanelExt, UiLabelExt, UiMenuItemExt, UiScrollViewExt,
    },
};

pub struct TabContainerPlugin;

impl Plugin for TabContainerPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            TabContainerUpdate
                .after(DraggableUpdate)
                .before(FloatingPanelUpdate),
        )
        .register_type::<Tab>()
        .add_systems(
            Update,
            (
                close_tab_on_context_menu_press,
                popout_tab_on_context_menu_press,
            )
                .after(MenuItemUpdate)
                .before(ContextMenuUpdate)
                .before(TabContainerUpdate),
        )
        .add_systems(
            Update,
            (
                apply_deferred,
                process_tab_viewport_content_removed
                    .run_if(should_process_tab_viewport_content_removed),
                process_tab_container_content_change,
                process_tab_viewport_content_change,
                update_tab_container_on_press,
                constrain_tab_container_active_tab,
                update_tab_container_on_change,
                handle_tab_dragging,
            )
                .chain()
                .in_set(TabContainerUpdate),
        );
    }
}

#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct TabContainerUpdate;

fn close_tab_on_context_menu_press(
    q_menu_items: Query<(Entity, &CloseTabContextMenu, &MenuItem), Changed<MenuItem>>,
    q_tab: Query<&Tab>,
    mut commands: Commands,
) {
    for (entity, tab_ref, menu_item) in &q_menu_items {
        if menu_item.interacted() {
            let Ok(tab) = q_tab.get(tab_ref.tab) else {
                warn!(
                    "Context menu tab reference {:?} refers to missing tab {:?}",
                    entity, tab_ref.tab
                );
                continue;
            };

            commands.entity(tab.panel).despawn_recursive();
        }
    }
}

fn popout_tab_on_context_menu_press(
    q_menu_items: Query<(Entity, &PopoutTabContextMenu, &MenuItem), Changed<MenuItem>>,
    q_tab: Query<(&Tab, &GlobalTransform)>,
    q_node: Query<&Node>,
    mut commands: Commands,
) {
    for (entity, tab_ref, menu_item) in &q_menu_items {
        if menu_item.interacted() {
            let Ok((tab, transform)) = q_tab.get(tab_ref.tab) else {
                warn!(
                    "Context menu tab reference {:?} refers to missing tab {:?}",
                    entity, tab_ref.tab
                );
                continue;
            };

            let Ok(container) = q_node.get(tab.container) else {
                warn!(
                    "Context menu tab reference {:?} refers to a tab without a container {:?}",
                    entity, tab_ref.tab
                );
                continue;
            };

            let size = container.size() * 0.8;
            let position = transform.translation().truncate();
            commands.add(PopoutPanelFromTabContainer {
                tab: tab_ref.tab,
                size,
                position,
            });
        }
    }
}

fn should_process_tab_viewport_content_removed(
    q_removed_children: RemovedComponents<Children>,
) -> bool {
    q_removed_children.len() > 0
}

fn process_tab_viewport_content_removed(
    q_tab_viewports: Query<(Entity, &TabViewport), Without<Children>>,
    q_tab_container: Query<&TabContainer>,
    q_children: Query<&Children>,
    q_tab: Query<&Tab>,
    mut commands: Commands,
) {
    for (entity, viewport) in &q_tab_viewports {
        let Ok(tab_container) = q_tab_container.get(viewport.container) else {
            error!("Missing tab container for viewport {:?}", entity);
            continue;
        };

        let Ok(tab_bar_children) = q_children.get(tab_container.bar) else {
            continue;
        };

        tab_bar_children
            .iter()
            .filter(|child| q_tab.get(**child).is_ok())
            .for_each(|child| commands.entity(*child).despawn_recursive());
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
                    .filter(|child| q_tab.get(**child).is_ok())
                    .map(|child| (*child, q_tab.get(*child).unwrap().panel))
                    .collect()
            } else {
                Vec::new()
            };

        let panels: Vec<(Entity, &Panel)> = children
            .iter()
            .filter(|child| q_panel.get(**child).is_ok())
            .map(|child| (*child, q_panel.get(*child).unwrap()))
            .collect();

        for (panel_id, panel) in &panels {
            if !tab_to_panel_ids.iter().any(|(_, p_id)| *p_id == *panel_id) {
                commands.ui_builder(tab_container.bar).tab(
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
            if tab_count > 0 {
                container.active = tab_count - 1;
            } else {
                container.active = 0;
            }
        }
    }
}

fn update_tab_container_on_change(
    q_tab_containers: Query<&TabContainer, Changed<TabContainer>>,
    q_tab: Query<(Entity, &Tab), With<Tab>>,
    q_children: Query<&Children>,
    mut q_panel: Query<&mut Panel>,
    mut commands: Commands,
) {
    for tab_container in &q_tab_containers {
        let Ok(tabs) = q_children.get(tab_container.bar) else {
            continue;
        };

        let flux_enabled = tabs.iter().filter(|tab| q_tab.get(**tab).is_ok()).count() > 1;
        for (i, id) in tabs.iter().enumerate() {
            if let Ok((tab_entity, tab)) = q_tab.get(*id) {
                commands
                    .style(tab_entity)
                    .flux_interaction_enabled(flux_enabled);

                if i == tab_container.active {
                    commands.style(tab_entity).background_color(Color::GRAY);
                    commands.style(tab.panel).show();

                    let Ok(mut panel) = q_panel.get_mut(tab.panel) else {
                        continue;
                    };
                    if !panel.visible {
                        panel.visible = true;
                    }
                } else {
                    commands.style(tab_entity).background_color(Color::NONE);
                    commands.style(tab.panel).hide();

                    let Ok(mut panel) = q_panel.get_mut(tab.panel) else {
                        continue;
                    };
                    if panel.visible {
                        panel.visible = false;
                    }
                }
            }
        }
    }
}

fn handle_tab_dragging(
    q_tabs: Query<(Entity, &Draggable, &Node, &Transform), (With<Tab>, Changed<Draggable>)>,
    q_tab_container: Query<&TabContainer>,
    q_tab_bar: Query<(&Interaction, &Node), With<TabBar>>,
    q_children: Query<&Children>,
    q_transform: Query<(&GlobalTransform, &Interaction)>,
    mut q_tab: Query<&mut Tab>,
    mut commands: Commands,
) {
    for (entity, draggable, node, transform) in &q_tabs {
        let tab = q_tab.get(entity).unwrap();

        let Ok(container) = q_tab_container.get(tab.container) else {
            warn!("Tried to drag orphan Tab {:?}", entity);
            continue;
        };

        let Ok((_bar_interaction, bar_node)) = q_tab_bar.get(container.bar) else {
            error!("Tab container {:?} doesn't have a tab bar", tab.container);
            continue;
        };

        let Ok(children) = q_children.get(container.bar) else {
            error!("Tab container has no tabs {:?}", tab.container);
            continue;
        };

        if children
            .iter()
            .filter(|child| q_tab.get(**child).is_ok())
            .count()
            < 2
        {
            continue;
        }

        let bar_half_width = bar_node.size().x / 2.;
        match draggable.state {
            DragState::DragStart => {
                children.iter().for_each(|child| {
                    if *child != entity && q_tab.get(*child).is_ok() {
                        commands.style(*child).disable_flux_interaction();
                    }
                });

                let Some(tab_index) = children
                    .iter()
                    .filter(|child| q_tab.get(**child).is_ok())
                    .position(|child| *child == entity)
                else {
                    error!("Tab {:?} isn't a child of its tab container bar", entity);
                    continue;
                };

                let left =
                    transform.translation.truncate().x - (node.size().x / 2.) + bar_half_width;
                let placeholder = commands
                    .ui_builder(container.bar)
                    .spawn(NodeBundle {
                        style: Style {
                            width: Val::Px(node.size().x * 1.1),
                            height: Val::Px(node.size().y),
                            ..default()
                        },
                        background_color: Color::NAVY.into(),
                        ..default()
                    })
                    .id();

                commands
                    .entity(container.bar)
                    .insert_children(tab_index, &[placeholder]);

                commands
                    .ui_builder(entity)
                    .style()
                    .position_type(PositionType::Absolute)
                    .left(Val::Px(left))
                    .z_index(ZIndex::Local(100));

                let mut tab = q_tab.get_mut(entity).unwrap();
                tab.placeholder = placeholder.into();
                tab.original_index = tab_index.into();
            }
            DragState::Dragging => {
                let Some(diff) = draggable.diff else {
                    continue;
                };
                let Some(position) = draggable.position else {
                    continue;
                };

                let Some(placeholder) = tab.placeholder else {
                    warn!("Tab {:?} missing placeholder", entity);
                    continue;
                };

                let new_x = transform.translation.truncate().x + diff.x + bar_half_width;
                let left = new_x - (node.size().x / 2.);
                let mut new_index: Option<usize> = None;
                let mut placeholder_index = children.len();
                for (i, child) in children.iter().enumerate() {
                    if *child == entity {
                        continue;
                    }
                    if *child == placeholder {
                        placeholder_index = i;
                        continue;
                    }
                    let Ok(_) = q_tab.get(entity) else {
                        continue;
                    };
                    let Ok((transform, interaction)) = q_transform.get(*child) else {
                        continue;
                    };

                    if *interaction == Interaction::Hovered {
                        if position.x < transform.translation().truncate().x {
                            if i < placeholder_index {
                                new_index = i.into();
                            } else {
                                // placeholder is between 0 and children.len or less
                                new_index = (i - 1).into();
                            }
                        } else {
                            if i + 1 < placeholder_index {
                                new_index = (i + 1).into();
                            } else {
                                // placeholder is between 0 and children.len or less
                                new_index = i.into();
                            }
                        }

                        break;
                    }
                }

                if let Some(new_index) = new_index {
                    commands
                        .entity(container.bar)
                        .insert_children(new_index, &[placeholder]);
                }

                commands.ui_builder(entity).style().left(Val::Px(left));
            }
            DragState::DragEnd => {
                children.iter().for_each(|child| {
                    if *child != entity && q_tab.get(*child).is_ok() {
                        commands.style(*child).enable_flux_interaction();
                    }
                });

                let Some(placeholder) = tab.placeholder else {
                    warn!("Tab {:?} missing placeholder", entity);
                    continue;
                };

                let Some(placeholder_index) =
                    children.iter().position(|child| *child == placeholder)
                else {
                    error!(
                        "Tab placeholder {:?} isn't a child of its tab container bar",
                        entity
                    );
                    continue;
                };

                commands
                    .style(entity)
                    .position_type(PositionType::Relative)
                    .left(Val::Auto)
                    .z_index(ZIndex::Local(0));

                commands
                    .entity(container.bar)
                    .insert_children(placeholder_index, &[entity]);

                commands.entity(placeholder).despawn_recursive();

                let mut tab = q_tab.get_mut(entity).unwrap();
                tab.placeholder = None;
                tab.original_index = None;
            }
            DragState::DragCanceled => {
                children.iter().for_each(|child| {
                    if *child != entity && q_tab.get(*child).is_ok() {
                        commands.style(*child).enable_flux_interaction();
                    }
                });

                let Some(placeholder) = tab.placeholder else {
                    warn!("Tab {:?} missing placeholder", entity);
                    continue;
                };

                let original_index = tab.original_index.unwrap_or(0);

                commands
                    .style(entity)
                    .position_type(PositionType::Relative)
                    .left(Val::Auto)
                    .z_index(ZIndex::Local(0));

                commands.entity(placeholder).despawn_recursive();

                commands
                    .entity(container.bar)
                    .insert_children(original_index, &[entity]);

                let mut tab = q_tab.get_mut(entity).unwrap();
                tab.placeholder = None;
                tab.original_index = None;
            }
            _ => continue,
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct CloseTabContextMenu {
    tab: Entity,
}

impl Default for CloseTabContextMenu {
    fn default() -> Self {
        Self {
            tab: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct PopoutTabContextMenu {
    tab: Entity,
}

impl Default for PopoutTabContextMenu {
    fn default() -> Self {
        Self {
            tab: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component, ContextMenuGenerator)]
pub struct Tab {
    container: Entity,
    panel: Entity,
    placeholder: Option<Entity>,
    original_index: Option<usize>,
}

impl Default for Tab {
    fn default() -> Self {
        Self {
            container: Entity::PLACEHOLDER,
            panel: Entity::PLACEHOLDER,
            placeholder: None,
            original_index: None,
        }
    }
}

impl ContextMenuGenerator for Tab {
    fn build_context_menu(&self, context: Entity, container: &mut UiBuilder<Entity>) {
        container
            .menu_item(MenuItemConfig {
                name: "Close Tab".into(),
                ..default()
            })
            .insert(CloseTabContextMenu { tab: context });
        container
            .menu_item(MenuItemConfig {
                name: "Popout Tab".into(),
                ..default()
            })
            .insert(PopoutTabContextMenu { tab: context });
    }

    fn placement_index(&self) -> usize {
        0
    }
}

struct PopoutPanelFromTabContainer {
    tab: Entity,
    size: Vec2,
    position: Vec2,
}

impl Command for PopoutPanelFromTabContainer {
    fn apply(self, world: &mut World) {
        let Ok(tab) = world.query::<&Tab>().get(world, self.tab) else {
            warn!("Cannot pop out panel from tab {:?}: Not a Tab", self.tab);
            return;
        };
        let tab_contaier_id = tab.container;

        let panel_id = tab.panel;
        let Ok(panel) = world.query::<&Panel>().get(world, panel_id) else {
            warn!("Cannot pop out panel {:?}: Not a Panel", panel_id);
            return;
        };
        let title = panel.title();

        let Ok((prev, flux, draggable, interaction)) = world
            .query::<(&PrevInteraction, &FluxInteraction, &Draggable, &Interaction)>()
            .get(world, self.tab)
        else {
            warn!("Failed to copy interaction states from {:?}", self.tab);
            return;
        };

        let bundle = (
            prev.clone(),
            flux.clone(),
            draggable.clone(),
            interaction.clone(),
        );

        let mut root_node = tab_contaier_id;
        while let Ok(parent) = world.query::<&Parent>().get(world, root_node) {
            root_node = parent.get();
        }

        let mut queue = CommandQueue::default();
        let mut commands = Commands::new(&mut queue, world);

        let mut container_id = Entity::PLACEHOLDER;
        let floating_panel_id = commands
            .ui_builder(root_node)
            .floating_panel(
                FloatingPanelConfig {
                    title: title.into(),
                    ..default()
                },
                FloatingPanelLayout {
                    size: self.size,
                    position: self.position.into(),
                    droppable: true,
                    ..default()
                },
                |container| {
                    container_id = container.id();
                },
            )
            .id();

        commands.entity(panel_id).set_parent(container_id);
        commands.entity(self.tab).despawn_recursive();
        queue.apply(world);

        let Ok(floating_panel) = world
            .query::<&FloatingPanel>()
            .get(world, floating_panel_id)
        else {
            error!(
                "Cannot find newly created floating panel {:?}",
                floating_panel_id
            );
            return;
        };

        let panel_title = floating_panel.title_container_id();
        world.entity_mut(panel_title).insert(bundle);
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

impl TabBar {
    pub fn container_id(&self) -> Entity {
        self.container
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
    fn base_tween() -> AnimationConfig {
        AnimationConfig {
            duration: 0.1,
            easing: Ease::OutExpo,
            ..default()
        }
    }

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
            InteractiveBackground {
                highlight: Color::rgba(0.9, 0.8, 0.7, 0.5).into(),
                ..default()
            },
            AnimatedInteraction::<InteractiveBackground> {
                tween: TabContainer::base_tween(),
                ..default()
            },
            Draggable::default(),
            GenerateContextMenu::default(),
        )
    }
}

pub trait UiTabContainerExt<'w, 's> {
    fn tab_container<'a>(
        &'a mut self,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<'w, 's, 'a, Entity>;
}

impl<'w, 's> UiTabContainerExt<'w, 's> for UiBuilder<'w, 's, '_, Entity> {
    fn tab_container<'a>(
        &'a mut self,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<'w, 's, 'a, Entity> {
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
    ) -> UiBuilder<'w, 's, 'a, Entity>;
}

impl<'w, 's> UiTabExt<'w, 's> for UiBuilder<'w, 's, '_, Entity> {
    fn tab<'a>(
        &'a mut self,
        title: String,
        tab_container: Entity,
        panel: Entity,
    ) -> UiBuilder<'w, 's, 'a, Entity> {
        self.container(
            (
                TabContainer::tab(),
                Tab {
                    container: tab_container,
                    panel,
                    ..default()
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
