use bevy::{ecs::system::EntityCommands, prelude::*, window::WindowResized};

use crate::{
    drag_interaction::{DragState, Draggable},
    TrackedInteraction,
};

use super::{hierarchy::MoveToParent, scroll_view::ScrollView};

pub struct FloatingPanelPlugin;

impl Plugin for FloatingPanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                index_floating_panel.run_if(panel_added),
                handle_window_resize,
                update_panel_on_title_drag,
                update_panel_layout,
            )
                .chain(),
        );
    }
}

fn panel_added(q_panels: Query<Entity, Added<FloatingPanel>>) -> bool {
    q_panels.iter().count() > 0
}

fn index_floating_panel(mut q_panels: Query<&mut FloatingPanel>) {
    for (i, mut panel) in &mut q_panels.iter_mut().enumerate() {
        panel.z_index = i + 1;
    }
}

fn update_panel_on_title_drag(
    q_draggable: Query<(&Draggable, &FloatingPanelTitle), Changed<Draggable>>,
    mut q_panel: Query<&mut FloatingPanel>,
) {
    for (draggable, panel_title) in &q_draggable {
        if draggable.state == DragState::Inactive
            || draggable.state == DragState::MaybeDragged
            || draggable.state == DragState::DragCanceled
        {
            continue;
        }

        let Ok(mut panel) = q_panel.get_mut(panel_title.panel) else {
            continue;
        };
        let Some(diff) = draggable.diff else {
            continue;
        };

        panel.position += diff;
    }
}

fn handle_window_resize(
    mut events: EventReader<WindowResized>,
    mut q_panels: Query<&mut FloatingPanel>,
) {
    for _ in events.read() {
        for mut panel in &mut q_panels {
            panel.position = panel.position;
        }
    }
}

fn update_panel_layout(
    mut q_panels: Query<(&FloatingPanel, &mut Style, &mut ZIndex), Changed<FloatingPanel>>,
) {
    for (panel, mut style, mut z_index) in &mut q_panels {
        style.width = Val::Px(panel.size.x);
        style.height = Val::Px(panel.size.y);
        style.left = Val::Px(panel.position.x);
        style.top = Val::Px(panel.position.y);
        *z_index = ZIndex::Local(panel.z_index as i32);
    }
}

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct FloatingPanel {
    pub size: Vec2,
    pub position: Vec2,
    pub z_index: usize,
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct FloatingPanelTitle {
    panel: Entity,
}

impl Default for FloatingPanelTitle {
    fn default() -> Self {
        Self {
            panel: Entity::PLACEHOLDER,
        }
    }
}

impl<'w, 's, 'a> FloatingPanel {
    pub fn spawn(
        parent: &'a mut ChildBuilder<'w, 's, '_>,
        title: String,
        size: Vec2,
        position: Option<Vec2>,
    ) -> EntityCommands<'w, 's, 'a> {
        let mut panel = parent.spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    width: Val::Px(size.x),
                    height: Val::Px(size.y),
                    border: UiRect::all(Val::Px(2.)),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Start,
                    ..default()
                },
                border_color: Color::BLACK.into(),
                background_color: Color::GRAY.into(),
                focus_policy: bevy::ui::FocusPolicy::Block,
                ..default()
            },
            FloatingPanel {
                size,
                position: position.unwrap_or_default(),
                z_index: 0,
            },
            MoveToParent { parent: None },
        ));

        let panel_id = panel.id();
        let mut container_id = Entity::PLACEHOLDER;
        panel.with_children(|parent| {
            parent
                .spawn((
                    ButtonBundle {
                        style: Style {
                            border: UiRect::right(Val::Px(2.)),
                            ..default()
                        },
                        border_color: Color::BLACK.into(),
                        background_color: Color::DARK_GRAY.into(),
                        ..default()
                    },
                    TrackedInteraction::default(),
                    FloatingPanelTitle { panel: panel_id },
                    Draggable::default(),
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        style: Style {
                            margin: UiRect::px(5., 5., 5., 2.),
                            ..default()
                        },
                        text: Text::from_section(title, TextStyle::default()),
                        ..default()
                    });
                });

            container_id = parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.),
                        height: Val::Percent(100.),
                        ..default()
                    },
                    ..default()
                })
                .id();
        });

        ScrollView::spawn_docked(parent, container_id.into())
    }
}
