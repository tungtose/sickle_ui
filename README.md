# Sickle UI

A widget library built on top of `bevy_ui`.

![Screenshot of the simple_editor example](/assets/images/Screenshot_Simple_Editor.png)

## Example

```
cargo build
cargo run --example simple_editor
```

**THIS IS CURRENTLY IN HEAVY DEVELOPMENT**

The project has no crate release yet. If you still want to try it locally from 
within your project, add a dependency on the repository directly.

Main missing features:
- Centralized focus management
- Text / Text area input widgets
- Documentation

What it can already do:
- Resizable layout
  - Rows / columns
  - Scroll views
  - Docking zones
  - Tab containers
  - Floating panels
  - Sized zones
  - Foldables
- Input
  - Slider
  - Dropdown
  - Checkbox
  - Radio groups
- Menu
  - Menu item (with leading/trailing icons and support for keyboard shortcuts)
  - Toggle menu item
  - Submenu
  - Context menu (component-based)
- Static
  - Icon
  - Label
- Utility
  - Command-based styling
  - Temporal tracking of interactions
  - Animated interactions
  - Context based extensions
  - Drag / drop interactions
  - Scroll interactions
- Theming
