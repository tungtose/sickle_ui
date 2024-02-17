pub mod checkbox;
pub mod column;
pub mod container;
pub mod context_menu;
pub mod docking_zone;
pub mod dropdown;
pub mod flexi_column;
pub mod flexi_row;
pub mod floating_panel;
pub mod label;
pub mod menu;
pub mod menu_item;
pub mod panel;
pub mod radio_group;
pub mod row;
pub mod scroll_view;
pub mod slider;
pub mod submenu;
pub mod tab_container;
pub mod toggle_menu_item;

use bevy::prelude::*;

use self::{
    checkbox::CheckboxPlugin, context_menu::ContextMenuPlugin, docking_zone::DockingZonePlugin,
    dropdown::DropdownPlugin, flexi_column::FlexiColumnPlugin, flexi_row::FlexiRowPlugin,
    floating_panel::FloatingPanelPlugin, menu::MenuPlugin, menu_item::MenuItemPlugin,
    radio_group::RadioGroupPlugin, scroll_view::ScrollViewPlugin, slider::InputSliderPlugin,
    submenu::SubmenuPlugin, toggle_menu_item::ToggleMenuItemPlugin,
};

pub mod prelude {
    pub use super::{
        checkbox::{Checkbox, UiCheckboxExt},
        column::{ColumnConfig, UiColumnExt},
        container::UiContainerExt,
        context_menu::{ContextMenuGenerator, GenerateContextMenu, ReflectContextMenuGenerator},
        docking_zone::{DockingZoneConfig, UiDockingZoneExt},
        dropdown::UiDropdownExt,
        flexi_column::{FlexiColumnConfig, UiFlexiColumnExt},
        flexi_row::{FlexiRowConfig, UiFlexiRowExt},
        floating_panel::{FloatingPanelConfig, FloatingPanelLayout, UiFloatingPanelExt},
        label::{LabelConfig, SetLabelTextExt, UiLabelExt},
        menu::{
            MenuConfig, MenuItemSeparator, MenuSeparator, UiMenuExt, UiMenuItemSeparatorExt,
            UiMenuSeparatorExt,
        },
        menu_item::{MenuItem, MenuItemConfig, MenuItemUpdate, UiMenuItemExt},
        radio_group::{RadioGroup, UiRadioGroupExt},
        row::{RowConfig, UiRowExt},
        scroll_view::{ScrollThrough, UiScrollViewExt},
        slider::{SliderConfig, UiSliderExt},
        submenu::{SubmenuConfig, UiSubmenuExt},
        toggle_menu_item::{ToggleMenuItem, ToggleMenuItemConfig, UiToggleMenuItemExt},
    };
}

pub struct WidgetsPlugin;

impl Plugin for WidgetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            CheckboxPlugin,
            ContextMenuPlugin,
            DockingZonePlugin,
            DropdownPlugin,
            FlexiColumnPlugin,
            FlexiRowPlugin,
            FloatingPanelPlugin,
            MenuPlugin,
            MenuItemPlugin,
            InputSliderPlugin,
            RadioGroupPlugin,
            ScrollViewPlugin,
            SubmenuPlugin,
            ToggleMenuItemPlugin,
        ));
    }
}
