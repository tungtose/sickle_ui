use bevy::{prelude::*, time::Stopwatch};
use bevy_reflect::Reflect;

pub struct FluxInteractionPlugin;

impl Plugin for FluxInteractionPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Update, FluxInteractionUpdate)
            .add_systems(
                Update,
                (
                    tick_flux_interaction_stopwatch,
                    update_flux_interaction,
                    reset_stopwatch_on_change,
                    update_prev_interaction,
                )
                    .chain()
                    .in_set(FluxInteractionUpdate),
            );
    }
}

#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct FluxInteractionUpdate;

#[derive(Bundle, Clone, Debug, Default)]
pub struct TrackedInteraction {
    pub interaction: FluxInteraction,
    pub stopwatch: FluxInteractionStopwatch,
    pub prev_interaction: PrevInteraction,
}

#[derive(Component, Clone, Copy, Debug, Default, Eq, PartialEq, Reflect)]
#[reflect(Component, PartialEq)]
pub enum FluxInteraction {
    #[default]
    None,
    PointerEnter,
    PointerLeave,
    Pressed,
    Released,
    PressCanceled,
}

#[derive(Component, Clone, Debug, Default)]
pub struct FluxInteractionStopwatch(pub Stopwatch);

#[derive(Component, Clone, Copy, Debug, Default, Eq, PartialEq, Reflect)]
#[reflect(Component, PartialEq)]
pub enum PrevInteraction {
    #[default]
    None,
    Pressed,
    Hovered,
}

fn tick_flux_interaction_stopwatch(
    time: Res<Time<Real>>,
    mut q_stopwatch: Query<&mut FluxInteractionStopwatch>,
) {
    for mut stopwatch in &mut q_stopwatch {
        stopwatch.0.tick(time.delta());
    }
}

fn update_flux_interaction(
    mut q_interaction: Query<
        (&PrevInteraction, &Interaction, &mut FluxInteraction),
        Changed<Interaction>,
    >,
) {
    for (prev, curr, mut flux) in &mut q_interaction {
        if *prev == PrevInteraction::None && *curr == Interaction::Hovered {
            *flux = FluxInteraction::PointerEnter;
        } else if *prev == PrevInteraction::None && *curr == Interaction::Pressed
            || *prev == PrevInteraction::Hovered && *curr == Interaction::Pressed
        {
            *flux = FluxInteraction::Pressed;
        } else if *prev == PrevInteraction::Hovered && *curr == Interaction::None {
            *flux = FluxInteraction::PointerLeave;
        } else if *prev == PrevInteraction::Pressed && *curr == Interaction::None {
            *flux = FluxInteraction::PressCanceled;
        } else if *prev == PrevInteraction::Pressed && *curr == Interaction::Hovered {
            *flux = FluxInteraction::Released;
        }
    }
}

fn reset_stopwatch_on_change(
    mut q_stopwatch: Query<&mut FluxInteractionStopwatch, Changed<FluxInteraction>>,
) {
    for mut stopwatch in &mut q_stopwatch {
        stopwatch.0.reset();
    }
}

fn update_prev_interaction(
    mut q_interaction: Query<(&mut PrevInteraction, &Interaction), Changed<Interaction>>,
) {
    for (mut prev_interaction, interaction) in &mut q_interaction {
        *prev_interaction = match *interaction {
            Interaction::Pressed => PrevInteraction::Pressed,
            Interaction::Hovered => PrevInteraction::Hovered,
            Interaction::None => PrevInteraction::None,
        };
    }
}
