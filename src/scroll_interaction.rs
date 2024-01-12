use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
};

pub struct ScrollInteractionPlugin;

impl Plugin for ScrollInteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_scrollables);
    }
}

fn update_scrollables(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    keys: Res<Input<KeyCode>>,
    mut q_scrollables: Query<(&mut Scrollable, &Interaction)>,
) {
    let mut axis = ScrollAxis::Vertical;
    let mut offset = 0.;
    let mut unit = MouseScrollUnit::Line;
    let mut has_event = false;

    // Only the last event is kept
    for mouse_wheel_event in mouse_wheel_events.read() {
        axis = ScrollAxis::Vertical;
        unit = mouse_wheel_event.unit;
        offset = if mouse_wheel_event.x != 0. {
            -mouse_wheel_event.x
        } else {
            -mouse_wheel_event.y
        };

        if mouse_wheel_event.x > 0. || keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
            axis = ScrollAxis::Horizontal;
        }

        has_event = true;
    }

    if !has_event {
        return;
    }

    for (mut scrollable, interaction) in &mut q_scrollables {
        if *interaction != Interaction::Hovered {
            continue;
        }
        
        scrollable.axis = Some(axis);
        scrollable.diff = offset.into();
        scrollable.unit = unit;

        break;
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Reflect)]
pub enum ScrollAxis {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct Scrollable {
    axis: Option<ScrollAxis>,
    diff: f32,
    unit: MouseScrollUnit,
}

impl Default for Scrollable {
    fn default() -> Self {
        Self {
            axis: Default::default(),
            diff: Default::default(),
            unit: MouseScrollUnit::Pixel,
        }
    }
}

impl Scrollable {
    pub fn last_change(&self) -> Option<(ScrollAxis, f32, MouseScrollUnit)> {
        let Some(axis) = self.axis else {
            return None;
        };

        Some((axis, self.diff, self.unit))
    }
}
