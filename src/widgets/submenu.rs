use bevy::{ecs::system::EntityCommands, prelude::*};

use crate::{
    ui_builder::{UiBuilder, UiBuilderExt},
    ui_commands::SetEntityDisplayExt,
    FluxInteraction, FluxInteractionUpdate, TrackedInteraction,
};

use super::{
    context_menu::ContextMenuUpdate,
    menu::{Menu, MenuUpdate},
    prelude::{MenuItemConfig, UiContainerExt, UiMenuItemExt},
};

const MENU_CONTAINER_Z_INDEX: i32 = 100001;
const MENU_CONTAINER_FADE_TIMEOUT: f32 = 1.;

pub struct SubmenuPlugin;

impl Plugin for SubmenuPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            SubmenuUpdate
                .after(FluxInteractionUpdate)
                .before(MenuUpdate)
                .before(ContextMenuUpdate),
        )
        .add_systems(
            Update,
            (
                update_submenu_timeout,
                open_submenu_on_hover,
                close_submenus_on_menu_change,
                update_open_submenu_containers,
                update_submenu_container_visibility,
            )
                .chain()
                .in_set(SubmenuUpdate),
        );
    }
}

#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct SubmenuUpdate;

fn update_submenu_timeout(
    r_time: Res<Time>,
    mut q_submenus: Query<(
        &mut SubmenuContainer,
        &mut SubmenuContainerState,
        &FluxInteraction,
    )>,
) {
    for (mut container, mut state, interaction) in &mut q_submenus {
        if *interaction == FluxInteraction::PointerEnter {
            state.is_locked = true;
        } else if !state.is_locked && state.timeout > 0. {
            state.timeout -= r_time.delta_seconds();
            if container.is_open && state.timeout < 0. {
                container.is_open = false;
            }
        }
    }
}

fn open_submenu_on_hover(
    q_submenus: Query<(Entity, &Submenu, &FluxInteraction), Changed<FluxInteraction>>,
    mut q_containers: Query<(&mut SubmenuContainer, &mut SubmenuContainerState)>,
) {
    for (entity, submenu, interaction) in &q_submenus {
        if *interaction == FluxInteraction::PointerEnter {
            let Ok((mut container, mut state)) = q_containers.get_mut(submenu.container) else {
                warn!("Submenu {:?} is missing its container", entity);
                continue;
            };

            container.is_open = true;
            state.is_locked = true;
            state.timeout = MENU_CONTAINER_FADE_TIMEOUT;
        } else if *interaction == FluxInteraction::PointerLeave {
            let Ok((_, mut state)) = q_containers.get_mut(submenu.container) else {
                warn!("Submenu {:?} is missing its container", entity);
                continue;
            };

            state.is_locked = false;
        }
    }
}

fn close_submenus_on_menu_change(
    q_menus: Query<Entity, Changed<Menu>>,
    mut q_submenus: Query<(&mut SubmenuContainer, &mut SubmenuContainerState)>,
) {
    let any_changed = q_menus.iter().count() > 0;
    if any_changed {
        for (mut container, mut state) in &mut q_submenus {
            container.is_open = false;
            state.is_locked = false;
            state.timeout = 0.;
        }
    }
}

fn update_open_submenu_containers(world: &mut World) {
    let mut q_all_containers = world.query::<(Entity, &mut SubmenuContainer)>();
    let mut q_changed =
        world.query_filtered::<(Entity, &SubmenuContainer), Changed<SubmenuContainer>>();

    let mut containers_closed: Vec<Entity> =
        Vec::with_capacity(q_all_containers.iter(&world).count());
    let mut sibling_containers: Vec<Entity> =
        Vec::with_capacity(q_all_containers.iter(&world).count());
    let mut open_container: Option<Entity> = None;
    let mut open_external: Option<Entity> = None;

    for (entity, container) in q_changed.iter(world) {
        if container.is_open {
            open_container = entity.into();
            open_external = container.external_container;
        } else {
            containers_closed.push(entity);
        }
    }

    if let Some(open) = open_container {
        for (entity, mut container) in q_all_containers.iter_mut(world) {
            if container.external_container == open_external && container.is_open && entity != open
            {
                container.is_open = false;
                sibling_containers.push(entity);
            }
        }
    }

    for entity in sibling_containers.iter() {
        close_containers_of(world, Some(*entity));
    }

    for entity in containers_closed.iter() {
        close_containers_of(world, Some(*entity));
    }
}

fn update_submenu_container_visibility(
    q_submenus: Query<(Entity, &SubmenuContainer), Changed<SubmenuContainer>>,
    mut commands: Commands,
) {
    for (entity, container) in &q_submenus {
        if container.is_open {
            commands.entity(entity).set_display(Display::Flex);
        } else {
            commands.entity(entity).set_display(Display::None);
        }
    }
}

fn close_containers_of(world: &mut World, external: Option<Entity>) {
    let mut q_all_containers = world.query::<(Entity, &mut SubmenuContainer)>();
    let mut containers_closed: Vec<Entity> =
        Vec::with_capacity(q_all_containers.iter(&world).count());

    for (entity, mut container) in q_all_containers.iter_mut(world) {
        if container.external_container == external && container.is_open {
            container.is_open = false;
            containers_closed.push(entity);
        }
    }

    for entity in containers_closed.iter() {
        close_containers_of(world, Some(*entity));
    }
}

#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct SubmenuContainerState {
    timeout: f32,
    is_locked: bool,
}

#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct SubmenuContainer {
    is_open: bool,
    external_container: Option<Entity>,
}

#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct SubmenuConfig {
    pub name: String,
    pub alt_code: Option<KeyCode>,
    pub leading_icon: Option<String>,
}

impl Into<MenuItemConfig> for SubmenuConfig {
    fn into(self) -> MenuItemConfig {
        MenuItemConfig {
            name: self.name,
            alt_code: self.alt_code,
            leading_icon: self.leading_icon,
            trailing_icon: "sickle://icons/submenu.png".to_string().into(),
            ..default()
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct Submenu {
    container: Entity,
}

impl Default for Submenu {
    fn default() -> Self {
        Self {
            container: Entity::PLACEHOLDER,
        }
    }
}

impl SubmenuContainer {
    fn frame() -> impl Bundle {
        (
            NodeBundle {
                style: Style {
                    left: Val::Percent(100.),
                    position_type: PositionType::Absolute,
                    border: UiRect::px(1., 1., 1., 1.),
                    padding: UiRect::px(5., 5., 5., 10.),
                    margin: UiRect::px(5., 0., -5., 0.),
                    flex_direction: FlexDirection::Column,
                    align_self: AlignSelf::FlexStart,
                    align_items: AlignItems::Stretch,
                    display: Display::None,
                    ..default()
                },
                z_index: ZIndex::Global(MENU_CONTAINER_Z_INDEX),
                background_color: Color::rgb(0.7, 0.6, 0.5).into(),
                border_color: Color::WHITE.into(),
                focus_policy: bevy::ui::FocusPolicy::Block,
                ..default()
            },
            Interaction::default(),
            TrackedInteraction::default(),
        )
    }
}

pub trait UiSubmenuExt<'w, 's> {
    fn submenu<'a>(
        &'a mut self,
        config: SubmenuConfig,
        spawn_items: impl FnOnce(&mut UiBuilder),
    ) -> EntityCommands<'w, 's, 'a>;
}

impl<'w, 's> UiSubmenuExt<'w, 's> for UiBuilder<'w, 's, '_> {
    fn submenu<'a>(
        &'a mut self,
        config: SubmenuConfig,
        spawn_items: impl FnOnce(&mut UiBuilder),
    ) -> EntityCommands<'w, 's, 'a> {
        let external_container = self.id();

        let menu_id = self.menu_item(config.clone().into()).id();
        let container = self
            .commands()
            .entity(menu_id)
            .ui_builder()
            .container(
                (
                    SubmenuContainer::frame(),
                    SubmenuContainerState::default(),
                    SubmenuContainer {
                        external_container,
                        ..default()
                    },
                ),
                spawn_items,
            )
            .id();

        self.commands()
            .entity(menu_id)
            .insert((Submenu { container }, config));

        self.commands().entity(menu_id)
    }
}
