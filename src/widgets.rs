pub mod checkbox;
pub mod dropdown;
pub mod floating_panel;
pub mod radio_group;
pub mod scroll_container;
pub mod slider;
pub mod hierarchy;

use bevy::prelude::*;

use self::{
    checkbox::CheckboxPlugin, dropdown::DropdownPlugin, floating_panel::FloatingPanelPlugin,
    radio_group::RadioGroupPlugin, scroll_container::ScrollContainerPlugin,
    slider::InputSliderPlugin,
};

pub struct WidgetsPlugin;

impl Plugin for WidgetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            CheckboxPlugin,
            DropdownPlugin,
            FloatingPanelPlugin,
            hierarchy::HierarchyPlugin,
            InputSliderPlugin,
            RadioGroupPlugin,
            ScrollContainerPlugin,            
        ));
    }
}
