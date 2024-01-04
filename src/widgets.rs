pub mod checkbox;

use bevy::{ecs::system::EntityCommands, prelude::*};

use self::checkbox::CheckboxPlugin;

pub struct WidgetsPlugin;

impl Plugin for WidgetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CheckboxPlugin);
    }
}

pub trait InputWidget<'w, 's, 'a> {
    fn spawn(builder: &'a mut ChildBuilder<'w, 's, '_>, label: Option<String>) -> EntityCommands<'w, 's, 'a>;
}
