use bevy::{ecs::system::CommandQueue, prelude::*};

use crate::prelude::*;

pub struct EntityComponentListPlugin;

impl Plugin for EntityComponentListPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EntityComponentTagPlugin)
            .add_systems(Update, update_entity_component_lists);
    }
}

fn update_entity_component_lists(world: &mut World) {
    let changed: Vec<(Entity, Option<Entity>)> = world
        .query::<(Entity, Ref<EntityComponentList>)>()
        .iter(world)
        .filter(|(_, list)| list.is_changed())
        .map(|(e, list_ref)| (e, list_ref.entity))
        .collect();

    for (container, selected_entity) in changed.iter().copied() {
        update_entity_component_list(container, selected_entity, world);
    }
}

fn update_entity_component_list(
    container: Entity,
    selected_entity: Option<Entity>,
    world: &mut World,
) {
    if let Some(selected) = selected_entity {
        if world.get_entity(selected).is_none() {
            let mut queue = CommandQueue::default();
            let mut commands = Commands::new(&mut queue, world);
            commands.entity(container).despawn_descendants();
            queue.apply(world);
            return;
        }

        let debug_infos: Vec<_> = world
            .inspect_entity(selected)
            .into_iter()
            .map(UiUtils::simplify_component_name)
            .collect();

        let mut queue = CommandQueue::default();
        let mut commands = Commands::new(&mut queue, world);

        // TODO: Maybe re-use existing tags if they exist
        commands.entity(container).despawn_descendants();
        let mut builder = commands.ui_builder(container);
        for info in debug_infos.iter().cloned() {
            builder.entity_component_tag(info);
        }
        queue.apply(world);
    } else {
        let mut queue = CommandQueue::default();
        let mut commands = Commands::new(&mut queue, world);
        commands.entity(container).despawn_descendants();
        queue.apply(world);
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct EntityComponentList {
    pub entity: Option<Entity>,
}

pub trait UiEntityComponentListExt {
    fn entity_component_list(&mut self, entity: Option<Entity>) -> UiBuilder<Entity>;
}

impl UiEntityComponentListExt for UiBuilder<'_, Entity> {
    fn entity_component_list(&mut self, entity: Option<Entity>) -> UiBuilder<Entity> {
        self.row(|row| {
            row.insert((
                Name::new("Entity Component List"),
                EntityComponentList { entity },
            ))
            .style()
            .overflow(Overflow::clip())
            .flex_wrap(FlexWrap::Wrap)
            .align_items(AlignItems::FlexStart)
            .align_content(AlignContent::FlexStart);
        })
    }
}

// TODO: Turn Tag into a standalone widget, use a theme override in the list container
pub struct EntityComponentTagPlugin;

impl Plugin for EntityComponentTagPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ComponentThemePlugin::<EntityComponentTag>::default());
    }
}

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct EntityComponentTag {
    label: Entity,
}

impl Default for EntityComponentTag {
    fn default() -> Self {
        Self {
            label: Entity::PLACEHOLDER,
        }
    }
}

impl DefaultTheme for EntityComponentTag {
    fn default_theme() -> Option<Theme<EntityComponentTag>> {
        EntityComponentTag::theme().into()
    }
}

impl UiContext for EntityComponentTag {
    fn get(&self, target: &str) -> Result<Entity, String> {
        match target {
            EntityComponentTag::LABEL => Ok(self.label),
            _ => Err(format!(
                "{} doesn't exists for EntityComponentTag. Possible contexts: {:?}",
                target,
                self.contexts()
            )),
        }
    }

    fn contexts(&self) -> Vec<&'static str> {
        vec![EntityComponentTag::LABEL]
    }
}

impl EntityComponentTag {
    pub const LABEL: &'static str = "Label";

    pub fn theme() -> Theme<EntityComponentTag> {
        let base_theme = PseudoTheme::deferred(None, EntityComponentTag::primary_style);
        Theme::new(vec![base_theme])
    }

    fn primary_style(style_builder: &mut StyleBuilder, theme_data: &ThemeData) {
        let theme_spacing = theme_data.spacing;
        let colors = theme_data.colors();
        let font = theme_data
            .text
            .get(FontStyle::Body, FontScale::Medium, FontType::Regular);

        style_builder
            .padding(UiRect::all(Val::Px(theme_spacing.gaps.small)))
            .margin(UiRect::all(Val::Px(theme_spacing.gaps.small)))
            .animated()
            .background_color(AnimatedVals {
                idle: colors.accent(Accent::Tertiary),
                enter_from: Color::NONE.into(),
                ..default()
            })
            .copy_from(theme_data.enter_animation);

        style_builder
            .switch_target(EntityComponentTag::LABEL)
            .sized_font(font)
            .font_color(colors.on(On::Tertiary));
    }

    fn frame() -> impl Bundle {
        (Name::new("Entity Component Tag"), NodeBundle::default())
    }
}

pub trait UiEntityComponentTagExt {
    fn entity_component_tag(&mut self, label: String) -> UiBuilder<Entity>;
}

impl UiEntityComponentTagExt for UiBuilder<'_, Entity> {
    fn entity_component_tag(&mut self, label: String) -> UiBuilder<Entity> {
        let mut tag = EntityComponentTag::default();
        let mut widget = self.container(EntityComponentTag::frame(), |container| {
            tag.label = container.label(LabelConfig { label, ..default() }).id();
        });

        widget.insert(tag);

        widget
    }
}
