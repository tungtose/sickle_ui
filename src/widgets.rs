pub mod checkbox;
pub mod radio_group;

use bevy::prelude::*;

use self::{checkbox::CheckboxPlugin, radio_group::RadioGroupPlugin};

pub struct WidgetsPlugin;

impl Plugin for WidgetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((CheckboxPlugin, RadioGroupPlugin));
    }
}
