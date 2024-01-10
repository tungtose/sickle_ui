pub mod checkbox;
pub mod dropdown;
pub mod radio_group;
pub mod scroll_container;
pub mod slider;

use bevy::prelude::*;

use self::{
    checkbox::CheckboxPlugin, dropdown::DropdownPlugin, radio_group::RadioGroupPlugin,
    scroll_container::ScrollContainerPlugin, slider::InputSliderPlugin,
};

pub struct WidgetsPlugin;

impl Plugin for WidgetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            CheckboxPlugin,
            DropdownPlugin,
            InputSliderPlugin,
            RadioGroupPlugin,
            ScrollContainerPlugin,
        ));
    }
}
