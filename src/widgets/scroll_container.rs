use bevy::{ecs::system::EntityCommands, prelude::*};

pub struct ScrollContainerPlugin;

impl Plugin for ScrollContainerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, move_inner_container);
    }
}

fn move_inner_container(
    q_to_move: Query<(Entity, &MoveToInnerContainer), Added<MoveToInnerContainer>>,
    mut q_scroll: Query<&mut ScrollContainer>,
    mut commands: Commands,
) {
    for (entity, to_move) in &q_to_move {
        let mut container = q_scroll.get_mut(to_move.scroll_container).unwrap();
        container.inner_container = entity;
        commands
            .entity(entity)
            .set_parent(to_move.inner_container)
            .remove::<MoveToInnerContainer>();
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct ScrollContainer {
    inner_container: Entity,
    horizontal_scroll: Entity,
    vertical_scroll: Entity,
}

impl Default for ScrollContainer {
    fn default() -> Self {
        Self {
            inner_container: Entity::PLACEHOLDER,
            horizontal_scroll: Entity::PLACEHOLDER,
            vertical_scroll: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Debug, Default, Eq, PartialEq, Reflect)]
pub enum ScrollAxis {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct ScrollBar {
    axis: ScrollAxis,
    container: Entity,
}

impl Default for ScrollBar {
    fn default() -> Self {
        Self {
            axis: Default::default(),
            container: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[component(storage = "SparseSet")]
#[reflect(Component)]
struct MoveToInnerContainer {
    scroll_container: Entity,
    inner_container: Entity,
}

impl Default for MoveToInnerContainer {
    fn default() -> Self {
        Self {
            scroll_container: Entity::PLACEHOLDER,
            inner_container: Entity::PLACEHOLDER,
        }
    }
}

impl<'w, 's, 'a> ScrollContainer {
    pub fn spawn(parent: &'a mut ChildBuilder<'w, 's, '_>) -> EntityCommands<'w, 's, 'a> {
        let mut inner_container_id = Entity::PLACEHOLDER;
        let mut container = parent.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                ..default()
            },
            ..default()
        });

        container.with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.),
                        height: Val::Percent(100.),
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    inner_container_id = parent
                        .spawn(NodeBundle {
                            style: Style {
                                height: Val::Px(160.),
                                width: Val::Percent(100.),
                                flex_direction: FlexDirection::Column,
                                overflow: Overflow::clip(),
                                ..default()
                            },
                            background_color: Color::DARK_GRAY.into(),
                            ..default()
                        })
                        .id();
                    parent.spawn(NodeBundle {
                        style: Style {
                            width: Val::Percent(100.),
                            height: Val::Px(20.),
                            ..default()
                        },
                        background_color: Color::AQUAMARINE.into(),
                        ..default()
                    });
                });

            parent.spawn(NodeBundle {
                style: Style {
                    width: Val::Px(20.),
                    height: Val::Percent(100.),
                    ..default()
                },
                background_color: Color::CYAN.into(),
                ..default()
            });
        });

        let scroll_container_id = container.insert(ScrollContainer { ..default() }).id();
        let inner_container = parent.spawn((
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    overflow: Overflow::clip(),
                    ..default()
                },
                ..default()
            },
            MoveToInnerContainer {
                scroll_container: scroll_container_id,
                inner_container: inner_container_id,
            },
        ));

        inner_container
    }
}
