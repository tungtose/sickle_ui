# Sickle UI

A widget library built on top of `bevy`'s internal `bevy_ui`.

![Screenshot of the simple_editor example](/assets/images/Screenshot_Simple_Editor.png)

## Example

If you clone the repository, you can simply build and run the main example:

```
cargo build
cargo run --example simple_editor
```

> [!WARNING]
> THIS IS CURRENTLY IN HEAVY DEVELOPMENT

The project has no crate release yet. If you still want to try it locally from 
within your project, add a dependency on the repository directly.

Main missing features:
- Centralized focus management
- Text / Text area input widgets

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
  - Material 3 based color scheme (dark/light, 3 contrast levels per theme)
  - Centralized sizing control
  - Centralized font control
  - Automatic theme updates
  - Theme overrides


## Getting started

First you need to add `sickle_ui` as a dependency to your project:

```toml
[dependencies]
sickle_ui = { rev = "a548517", git = "https://github.com/UmbraLuminosa/sickle_ui" }
```

> [!NOTE]
> change `rev = "..."` to a version of your chosing. Major versions are marked with a git tag.

Once you have the new dependency, `cargo build` to download it. Now you are ready to use it, 
so add it to your app as a plugin:

```rust
use bevy::prelude::*;
use sickle_ui::{prelude::*, SickleUiPlugin};

fn main() {
  App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(SickleUiPlugin)
        // ... your actual app plugins and systems can go here of course
        .run();
}
```

The main `SickleUiPlugin` takes care of adding all the convenient features `sickle_ui` offers, and
the `sickle_ui::prelude::*` brings into scope all available extensions. Have a look at the `simple_editor` 
example (that is displayed in the screenshot above) for how different parts work together.


## Foreword

> [!IMPORTANT] 
> Sickle UI is primarily using `Commands` and `EntityCommands` to spawn, style, and configure widgets.
> Systems using these widgets need to consider that the changes will not be reflected in the ECS `world`
> until the next `apply_deferred` is executed. This is mostly automatic starting from `bevy 0.13`. Internally
> `sickle_ui` uses systems in well defined sets and order to make sure all widgets play nicely with each other.


## Basic use case

In the most simple use cases you just want to use existing widgets to build up your UI. Sickle UI adds 
extensions to both `Commands` and `EntityCommands`, so in a regular system context you can quickly
create a layout by calling a chain of functions. Comparing vanilla and Sickle UI:


### Vanilla `bevy`

In Bevy, you can use `commands.spawn(bundle)` and `commands.entity(entity).with_children(builder)` to 
spawn your entities. Typically, you would pass in a `NodeBundle`, `ButtonBundle`, or perhaps an 
`ImageBundle` just to name a few. Then, you can use the `.with_children(builder)` extension to spawn
sub-entities. This will quickly become verbose and convulated with Rust's borrowing rules. It will 
be difficult to create entities with two way references between parent and children, elements further
down the tree, or siblings.

```rust
fn setup(mut commands: Commands) {
  commands.spawn(NodeBundle {
      style: Style {
          height: Val::Percent(100.),
          flex_direction: FlexDirection::Column,
          ..default()
      },
      background_color: Color::NONE.into(),
      ..default()
  }).with_children(|parent|{
    parent.spawn(NodeBundle::default()).with_children(|parent|{
      // ...
    });
  });
}
```


### Sickle UI

The library takes care of this by abstracting widget creation behind builder extensions such as:

```rust
fn setup(mut commands: Commands) {
  commands.ui_builder(UiRoot).column(|column|{
    column.row(|row|{
      // ... etc.
    });
  });
}
```

While this may seem as a simple shorthand, the key difference is that `column` and `row` in the 
callbacks are contextual builders themselves and they give you access to `commands` and, where
available, `entity_commands`. You can easily jump to another entity to insert components,
style, or spawn new sub-entitites without tripping Rust's borrow checker.


### Did I mention style?

Yes, you can also style entities spawned by the command chains, as simple as:

```rust
fn setup(mut commands: Commands) {
  commands.ui_builder(UiRoot).column(|column|{
    // ...
  })
  .style()
  .width(Val::Percent(100.));
}
```

> [!NOTE]
> The return value of a builder function can be different from the internal builder. A
> good example would be `scroll_view`, where the external return value is the builder of the frame
> (outermost) entity, while the internal builder is for its content view (that will be clipped to
> the frame!).

This means that in _some_ cases, this also works as expected:

```rust
fn setup(mut commands: Commands) {
  commands.ui_builder(UiRoot).column(|column|{
    column
      .style()
      .width(Val::Percent(100.));
  });
}
```

The difference in the above is marely a styling choice of the developer (pun intended).

> [!IMPORTANT] 
> Styling is applied as a regular command in the chain, so rendering of the component
> will change the next time UI layout is calulated by `bevy` in its `PostUpdate` systems. The `style`
> commands are mapped to the `Style` component fields and _some other_ component fields that affect
> the overall display of the `Node`, such as `BackgroundColor`, `BorderColor`, etc.

> [!WARNING]
> Theming may override styles applied this way. Read [Theming](#theming) further down on how theming works.


### Noteworthy contexts

As mentioned, all builder function have a context. 
- The root one is `UiRoot`. The entity spawned in the `UiRoot` context does not have a `Parent` entity,
hence it will be a root `Node`.
- The most common regular context is `Entity`, which can be acquired by calling `commands.ui_builder(entity)`.
Where `entity` is an entity - ID - acquired by some other means, such as spawing or querying.

> [!TIP]
> Other contexts are specific for use cases, such as the tab container's or that of the menu system. You'll
> find these eventually as you use these widgets, but they are generally transparent. Use the editor's 
> auto-complete feature to see what extensions are available in each!

> [!CAUTION]
> `UiRoot` must not be confused with `UiContextRoot`. The former is a marker to indicate that we spawn without
> a `Parent` while the later is a component that indicates the _logical_ root of a sub-tree of widgets.
> It is used by widgets such as `ContextMenu` and `TabContainer` to find mounting points for dynamically
> spawned widgets. `ContextMenu` places the menu container at `UiContextRoot` and `TabContainer` creates
> the `FloatingPanel` at this location in the tree when a tab is popped out.


### Alright, now I want to find my `column`

Fear not your column, or any other widget you create can be used like any other entity you have around.
To just add a component:

```rust
fn setup(mut commands: Commands) {
  commands.ui_builder(UiRoot).column(|_|{}).insert(MyMarkerComponent);
}
```

You could capture its ID as well:

```rust
fn setup(mut commands: Commands) {
  let my_column = commands.ui_builder(UiRoot).column(|_|{}).id();

  // ... OR

  let my_column = commands.ui_builder(UiRoot).column(|_|{}).insert(MyMarkerComponent).id();
}
```

> [!TIP]
> The same applies here as with styling. Callbacks may point to the same entity as the frame, so
> `insert` may be called in the callback as well:

```rust
fn setup(mut commands: Commands) {
  commands.ui_builder(UiRoot).column(|column|{
    column.insert(MyMarkerComponent); 
  });
}
```


### OK, but I didn't find a widget I need

If you just need a simple bundle somewhere in the tree, you can either use `spawn` or a container widget, like `container` to create or chain your one-off node. So, converting the `bevy` example we started with:

```rust
fn setup(mut commands: Commands) {
  commands.ui_builder(UiRoot).column(|column|{
    column.container(NodeBundle {
        style: Style {
            height: Val::Percent(100.),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        background_color: Color::NONE.into(),
        ..default()
    }, |my_container|{
      // ... etc. my_container is an `Entity` context UiBuilder
    });
  });  
}
```

If you do not even need to spawn children for this widget:

```rust
fn setup(mut commands: Commands) {
  commands.ui_builder(UiRoot).column(|column|{
    column.spawn(NodeBundle {
        style: Style {
            height: Val::Percent(100.),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        background_color: Color::NONE.into(),
        ..default()
    });
  });
}
```

> [!TIP]
> Since we are using `Commands` and `EntityCommands` and just spawn regular `bevy_ui` `Node`s
> you can also mix this syntax with the vanilla Bevy spawns:

```rust
fn setup(mut commands: Commands) {
  let mut inner_id = Entity::PLACEHOLDER;
  
  commands.spawn(NodeBundle {
      style: Style {
          height: Val::Percent(100.),
          flex_direction: FlexDirection::Column,
          ..default()
      },
      background_color: Color::NONE.into(),
      ..default()
  }).with_children(|parent|{
    inner_id = parent.spawn(NodeBundle::default()).with_children(|parent|{
      // ...
    }).id();
  });

  commands.ui_builder(inner_id).column(|column|{
    // Add a column into the inner entity and continue.
  });
}
```

> [!TIP]
> And vica-versa!

```rust
fn setup(mut commands: Commands) {
  commands.ui_builder(UiRoot).column(|column|{
    column.row(|row|{
      let mut row_commands = row.entity_commands();
      row_commands.with_children(|parent| {
        // ... etc.
      });
    });
  });
}
```


### OK, but my widget isn't _simple_

Then you shall move on to the next section, [Extending Sickle UI](#extending-sickle-ui)!


## Extending Sickle UI

Sickle UI can be extended on multiple levels. Starting from the most simple one:

- Structural extensions
- Functional extensions
- Themed widgets
- Contextually themed widgets

These are however, NOT distinct extensions. Rather these are levels of customization you can apply
to the widgets you create. If you don't need dynamic theming, you don't need to implement all that.

> [!TIP]
> `sickle_ui` includes a snipped for each of the scenarios outlined above to get you started.
> These are VSCode snippets, available in the `.vscode` folder. You can either copy the
> `sickle_ui.code-snippets` to your workspace's `.vscode` folder, or copy the file contents to your
> Rust snippets (`File` -> `Preferences` -> `Configure User Snippets` -> `[select the rust language from the list]`)


### Structural extensions

These are widgets that don't need systems and just create a pre-defined sub-tree that you can easily inject
in the contexts you define them in. In this case you just need to create the relevant extension and describe
your plugin structure using the technique described under [OK, but I didn't find a widget I need](#ok-but-i-didnt-find-a-widget-i-need)

An example of this would be:

```rust
#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct MyWidget;

impl MyWidget {
    fn frame() -> impl Bundle {
        (Name::new("My Widget"), NodeBundle::default())
    }
}

pub trait UiMyWidgetExt {
    fn my_widget(
        &mut self,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<Entity>;
}

impl UiMyWidgetExt for UiBuilder<'_, Entity> {
    fn my_widget(
        &mut self,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<Entity> {
        self.container((MyWidget::frame(), MyWidget), spawn_children)
    }
}
```

> [!TIP]
> The above has been generated with the snippet `Sickle UI Widget` available if you start typing `sickle` in
> an `.rs` file in VSCode (if you have added the snippets). You can customize the suggestion trigger in the
> snippet files, but it is recommended to avoid using `widget` as a trigger (it collides with often used `width`).

> [!TIP]
> The snippets support 3 tab points: The widget component name, the convenience `Name` component string, 
> and the actual extension function name.

You can then use your widget after bringing it into scope:

```rust
use my_widget::UiMyWidgetExt;

fn setup(mut commands: Commands) {
  commands.ui_builder(UiRoot).my_widget(|my_widget|{
    // ... do more here!
  });
}
```

You may have noticed that the snippet extends the `Entity` context of `UiBuilder`. Your widget will be
available in these contexts, provided you add the `use my_widget::UiMyWidgetExt;` to bring it in scope.

> [!TIP]
> VSCode with the regular Rust extensions is smart enouth to _suggest_ the import if you type out the
> extension name and press `Ctrl + .` (or the Mac equivalent `Command + .`).

You may have _also_ noticed that the snippet uses `self` to spawn the `container`. `self` will simply be
a `UiBuilder` of the `Entity` context, so any _other_ extensions that you brought into scope with `use`
will be available. This also means that `style` commands are also available, so long as you have imported them.

## Theming