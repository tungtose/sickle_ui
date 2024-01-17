pub mod checkbox;
pub mod dropdown;
pub mod floating_panel;
pub mod hierarchy;
pub mod radio_group;
pub mod scroll_view;
pub mod slider;

use bevy::prelude::*;

use self::{
    checkbox::CheckboxPlugin, dropdown::DropdownPlugin, floating_panel::FloatingPanelPlugin,
    radio_group::RadioGroupPlugin, scroll_view::ScrollViewPlugin, slider::InputSliderPlugin,
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
            ScrollViewPlugin,
        ));
    }
}
