pub mod checkbox;

use bevy::prelude::*;

use self::checkbox::CheckboxPlugin;

pub struct WidgetsPlugin;

impl Plugin for WidgetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CheckboxPlugin);
    }
}

pub trait InputWidget {
    fn spawn(builder: &mut ChildBuilder, label: Option<String>);
}
