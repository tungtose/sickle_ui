pub mod checkbox;
pub mod dropdown;
pub mod radio_group;
pub mod slider;

use bevy::prelude::*;

use self::{
    checkbox::CheckboxPlugin, dropdown::InputDropdownPlugin, radio_group::RadioGroupPlugin,
    slider::InputSliderPlugin,
};

pub struct WidgetsPlugin;

impl Plugin for WidgetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            CheckboxPlugin,
            InputDropdownPlugin,
            InputSliderPlugin,
            RadioGroupPlugin,
        ));
    }
}
