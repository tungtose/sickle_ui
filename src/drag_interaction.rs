use bevy::{prelude::*, window::PrimaryWindow};
use bevy_reflect::Reflect;

use crate::{FluxInteraction, FluxInteractionUpdate};

pub struct DragInteractionPlugin;

impl Plugin for DragInteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update_drag_progress, update_drag_state)
                .chain()
                .after(FluxInteractionUpdate),
        );
    }
}

#[derive(Component, Default, Debug, Reflect)]
#[reflect(Component)]
pub struct Draggable {
    pub state: DragState,
    pub origin: Option<Vec2>,
    pub position: Option<Vec2>,
    pub diff: Option<Vec2>,
    pub source: DragSource,
}

#[derive(Default, Debug, PartialEq, Eq, Reflect)]
#[reflect]
pub enum DragState {
    #[default]
    Inactive,
    MaybeDragged,
    DragStart,
    Dragging,
    DragEnd,
    DragCanceled,
}

#[derive(Default, Debug, PartialEq, Eq, Reflect)]
#[reflect]
pub enum DragSource {
    #[default]
    Mouse,
    Touch(u64),
}

// fn debug_drag_state(q_draggable: Query<&Draggable, Changed<Draggable>>) {
//     for draggable in &q_draggable {
//         println!("{:?}", draggable);
//     }
// }

fn update_drag_progress(
    mut q_draggable: Query<(&mut Draggable, &FluxInteraction)>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    r_touches: Res<Touches>,
) {
    for (mut draggable, flux_interaction) in &mut q_draggable {
        if draggable.state == DragState::DragEnd {
            draggable.state = DragState::Inactive;
            draggable.origin = None;
            draggable.position = None;
            draggable.diff = Some(Vec2::default());
        } else if *flux_interaction == FluxInteraction::Pressed
            && (draggable.state == DragState::MaybeDragged
                || draggable.state == DragState::DragStart
                || draggable.state == DragState::Dragging)
        {
            // Drag start is only a single frame, triggered after initial movement
            if draggable.state == DragState::DragStart {
                draggable.state = DragState::Dragging;
            }

            let position: Option<Vec2> = match draggable.source {
                DragSource::Mouse => match q_window.single().cursor_position() {
                    Some(pos) => Some(pos),
                    None => None,
                },
                DragSource::Touch(id) => match r_touches.get_pressed(id) {
                    Some(touch) => Some(touch.position()),
                    None => None,
                },
            };

            if let (Some(current_position), Some(new_position)) = (draggable.position, position) {
                let diff = new_position - current_position;

                // No tolerance threshold, just move
                if diff.length_squared() > 0. {
                    if draggable.state == DragState::MaybeDragged {
                        draggable.state = DragState::DragStart;
                    }

                    draggable.position = new_position.into();
                    draggable.diff = Some(new_position - current_position);
                }
            }
        }
    }
}

fn update_drag_state(
    mut q_draggable: Query<(&mut Draggable, &FluxInteraction), Changed<FluxInteraction>>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    r_touches: Res<Touches>,
) {
    for (mut draggable, flux_interaction) in &mut q_draggable {
        if *flux_interaction == FluxInteraction::Pressed
            && draggable.state != DragState::MaybeDragged
        {
            let mut drag_source = DragSource::Mouse;
            let mut position = q_window.single().cursor_position();
            if position.is_none() {
                position = r_touches.first_pressed_position();
                drag_source = DragSource::Touch(r_touches.iter().next().unwrap().id());
            }

            draggable.state = DragState::MaybeDragged;
            draggable.source = drag_source;
            draggable.origin = position;
            draggable.position = position;
            draggable.diff = Some(Vec2::default());
        } else if *flux_interaction == FluxInteraction::Released
            || *flux_interaction == FluxInteraction::PressCanceled
        {
            if draggable.state == DragState::DragStart || draggable.state == DragState::Dragging {
                draggable.state = DragState::DragEnd;
            } else if draggable.state == DragState::MaybeDragged {
                draggable.state = DragState::Inactive;
                draggable.origin = None;
                draggable.position = None;
                draggable.diff = Some(Vec2::default());
            }
        }
    }
}
