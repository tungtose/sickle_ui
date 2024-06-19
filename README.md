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

The difference in the above is merely a styling choice of the developer (pun intended).

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
> a `Parent` while the latter is a component that indicates the _logical_ root of a sub-tree of widgets.
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
> `sickle_ui` includes a snippet for each of the scenarios outlined above to get you started.
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


### Functional extension

Functional extension simply means that your widget _does_ something beyond creating a pre-defined structure.
You can use the snippet `Sickle UI plugin widget` to generate code similar to the one outlined in [Structural extensions](#structural-extensions), with the addition of a plugin:

```rust
pub struct MyWidgetPlugin;

impl Plugin for MyWidgetPlugin {
    fn build(&self, _app: &mut App) {
        // TODO
    }
}

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
> The snippets also supports tab points, so you can quickly name the widget and plugin in a consistent manner.

All that is left is for you to implement the heart of the widget and the systems that act on it. Don't
forget to add the generated plugin to your app!


### Themed widgets

Now, this is where the fun begins.

Themed widgets refer to widgets that have a style defined for them in a central place. However, themed widgets 
also allow overrides to their style, based on their position in the widget tree or their [pseudo states](#pseudo-states).

> [!IMPORTANT]
> Themed widgets only apply style to their outermost `Node`, but not to their sub-nodes. Those are the 
> [Contextually themed widgets](#contextually-themed-widgets).

Similarly to the previous cases, there is a snippet to generate the shell of a themed widget: 
The `Sickle UI themed plugin widget`.

> [!TIP]
> The snippets also supports tab points, so you can quickly name the widget and plugin in a consistent manner.

```rust
pub struct MyWidgetPlugin;

impl Plugin for MyWidgetPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ComponentThemePlugin::<MyWidget>::default());
    }
}

#[derive(Component, Clone, Debug, Default, Reflect, UiContext)]
#[reflect(Component)]
pub struct MyWidget;

impl DefaultTheme for MyWidget {
    fn default_theme() -> Option<Theme<MyWidget>> {
        MyWidget::theme().into()
    }
}

impl MyWidget {
    pub fn theme() -> Theme<MyWidget> {
        let base_theme = PseudoTheme::deferred(None, MyWidget::primary_style);
        Theme::new(vec![base_theme])
    }

    fn primary_style(style_builder: &mut StyleBuilder, theme_data: &ThemeData) {
        let theme_spacing = theme_data.spacing;
        let colors = theme_data.colors();

        style_builder
            .background_color(colors.surface(Surface::Surface))
            .padding(UiRect::all(Val::Px(theme_spacing.gaps.small)));
    }

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

While we have seen most of the above from the previous snippets, there are a couple additions.

First, an additional plugin has been injected to our app in the widget's plugin definition:

```rust
impl Plugin for MyWidgetPlugin {
    fn build(&self, app: &mut App) {
        // This here is very important!
        app.add_plugins(ComponentThemePlugin::<MyWidget>::default());
    }
}
```

The `ComponentThemePlugin` handles theme calculation and reloading for the component is is added for. 
In this case we added it for `MyWidget`, which is the example component.

> [!IMPORTANT]
> `MyWidget` now ***must*** derives `UiContext`. This derive provides default implementation for the context 
> we will look at later in [Contextually themed widgets](#contextually-themed-widgets).

Next, we have the implementation of `DefaultTheme`:

```rust
impl DefaultTheme for MyWidget {
    fn default_theme() -> Option<Theme<MyWidget>> {
        MyWidget::theme().into()
    }
}
```

This is the theme that will be applied (unless it returns `None`) to any widget in the widget tree that has no
overrides on any of its ancestors. We will look at how this works exactly in the [Theming](#theming) section.

For now, the key point is that it is generally desirable to implement the default theme of the widget as part
of this implementation so an explicit injection is not needed or a sane fallback is provided.

The last part is the actual definition of the theme as part of the widget's `impl` block:

```rust
impl MyWidget {
    pub fn theme() -> Theme<MyWidget> {
        let base_theme = PseudoTheme::deferred(None, MyWidget::primary_style);
        Theme::new(vec![base_theme])
    }

    fn primary_style(style_builder: &mut StyleBuilder, theme_data: &ThemeData) {
        let theme_spacing = theme_data.spacing;
        let colors = theme_data.colors();

        style_builder
            .background_color(colors.surface(Surface::Surface))
            .padding(UiRect::all(Val::Px(theme_spacing.gaps.small)));
    }

    // ...
}
```

The two function above define the theme itself and the styling that is applied as part of the
[PseudoTheme](#pseudo-theme) of `None`. This is simply the style that is applied when the widget has no special
[PseudoState](#pseudo-states) attached to it. It is the base theme and the fallback style that is always applied
to any new entities that are added to the widget tree. It is also the basis of any overrides.

In the simplest use case, defining the style is just a matter of calling style function on the
provided `style_builder`. The methods availbale here are the same as the ones provided by the `UiStyle`
extensions outlined in [Did I mention style?](#did-i-mention-style) with a few additions.

> [!TIP]
> See [Style builder](#style-builder) further below for information on what it provides.

With this, we have a convenient place to implement all our styling needs.

> [!IMPORTANT]
> Styles defined in a theme are applied in `PostUpdate` as part of the `DynamicStylePostUpdate` system set.
> This means that any style the node was created with (as overrides in the spawn bundle) or those that were
> applied via `.style()` commands will potentially be overwritten here.


### Contextually themed widgets

Contextually themed widgets take [Themed widgets](#themed-widgets) a step further by allowing the styling to be
applied to sub-widgets defined as part of the main widget. The snippet `Sickle UI contexted themed plugin widget`
generates the following shell:

```rust
pub struct MyWidgetPlugin;

impl Plugin for MyWidgetPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ComponentThemePlugin::<MyWidget>::default());
    }
}

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct MyWidget {
    label: Entity,
}

impl Default for MyWidget {
    fn default() -> Self {
        Self {
            label: Entity::PLACEHOLDER,
        }
    }
}

impl DefaultTheme for MyWidget {
    fn default_theme() -> Option<Theme<MyWidget>> {
        MyWidget::theme().into()
    }
}

impl UiContext for MyWidget {
    fn get(&self, target: &str) -> Result<Entity, String> {
        match target {
            MyWidget::LABEL => Ok(self.label),
            _ => Err(format!(
                "{} doesn't exists for MyWidget. Possible contexts: {:?}",
                target,
                self.contexts()
            )),
        }
    }

    fn contexts(&self) -> Vec<&'static str> {
        vec![MyWidget::LABEL]
    }
}

impl MyWidget {
    pub const LABEL: &'static str = "Label";

    pub fn theme() -> Theme<MyWidget> {
        let base_theme = PseudoTheme::deferred(None, MyWidget::primary_style);
        Theme::new(vec![base_theme])
    }

    fn primary_style(style_builder: &mut StyleBuilder, theme_data: &ThemeData) {
        let theme_spacing = theme_data.spacing;
        let colors = theme_data.colors();
        let font = theme_data
            .text
            .get(FontStyle::Body, FontScale::Medium, FontType::Regular);

        style_builder
            .background_color(colors.surface(Surface::Surface))
            .padding(UiRect::all(Val::Px(theme_spacing.gaps.small)));

        style_builder
            .switch_target(MyWidget::LABEL)
            .sized_font(font);
    }

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
        let label = self
            .label(LabelConfig {
                label: "MyWidget".into(),
                ..default()
            })
            .id();

        self.container((MyWidget::frame(), MyWidget { label }), spawn_children)
    }
}
```

> [!TIP]
> The snippets also supports tab points, so you can quickly name the widget and plugin in a consistent manner.

Now, our widget component is no longer just a tag. It now has a reference to a label sub-widget:

```rust
#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct MyWidget {
    label: Entity,
}

impl Default for MyWidget {
    fn default() -> Self {
        Self {
            label: Entity::PLACEHOLDER,
        }
    }
}
```

We need to implement `Default` for it manually, since `Entity` has no default. Using `Entity::PLACEHOLDER` is
alright as long as we make sure we always assign an actual entity to it (otherwise it will panic!).

But this isnt't the only addition. Now our sippet defined an implementation for `UiContext` we previously
got from a simple `derive`:

```rust
impl UiContext for MyWidget {
    fn get(&self, target: &str) -> Result<Entity, String> {
        match target {
            MyWidget::LABEL => Ok(self.label),
            _ => Err(format!(
                "{} doesn't exists for MyWidget. Possible contexts: {:?}",
                target,
                self.contexts()
            )),
        }
    }

    fn contexts(&self) -> Vec<&'static str> {
        vec![MyWidget::LABEL]
    }
}
```

This tells the theming system that `MyWidget` has a single additional context (besides the main entity).
The additional context can be accessed by the `MyWidget::LABEL` constant, which was added to the `ìmpl` block:

```rust 
impl MyWidget {
    pub const LABEL: &'static str = "Label";

    // ...
}
```

Further down we can also see a change: The `primary_style` now applies styling to the label!

```rust
impl MyWidget {    
    // ...
    
    fn primary_style(style_builder: &mut StyleBuilder, theme_data: &ThemeData) {
        let theme_spacing = theme_data.spacing;
        let colors = theme_data.colors();
        let font = theme_data
            .text
            .get(FontStyle::Body, FontScale::Medium, FontType::Regular);

        style_builder
            .background_color(colors.surface(Surface::Surface))
            .padding(UiRect::all(Val::Px(theme_spacing.gaps.small)));

        style_builder
            .switch_target(MyWidget::LABEL)
            .sized_font(font);
    }

    // ...    
}
```

In the above code, there is a call on `style_builder` to `switch_target` to our label and set its font size.
Refer to [Style builder](#style-builder) for how this works in detail.

> [!CAUTION]
> All subsequent calls to `style_builder` will be applied to the target. You can `reset_target` on the builder
> to swap to the main widget again, but it is more readable to have each target in a single chain / group. 


### That's it?

In a nutshell, yes. If you use the snippets, you can quickly set up a complex widget tree and define each
sub-widget's style by chaining the calls to the `style_builder`. Of course, there are other ways to interact
with the theming process, such as accesing the world or the current widget component, but the heart of it is
the same: A theme, made up of pseudo themes that build the styling of the widget and its sub-widgets.


## Theming

### Pseudo theme

### Pseudo states

### Style builder

### Dynamic style