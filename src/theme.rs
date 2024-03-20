use std::marker::PhantomData;

use bevy::{prelude::*, utils::HashSet};
use sickle_math::lerp::Lerp;

use crate::{
    animated_interaction::AnimatedInteraction,
    interactions::{ComponentController, InteractiveBackground},
};

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum StylableAttribute {
    Display,
    PositionType,
    Overflow,
    Direction,
    Left,
    Right,
    Top,
    Bottom,
    Width,
    Height,
    MinWidth,
    MinHeight,
    AspectRation,
    AlignItems,
    JustifyItems,
    AlignSelf,
    JustifySelf,
    AlignContent,
    JustifyContent,
    Margin,
    Padding,
    Border,
    FlexDirection,
    FlexWrap,
    FlexGrow,
    FlexShrink,
    FlexBasis,
    RowGap,
    ColumnGap,
    GridAutoFlow,
    GridTemplateRows,
    GridTemplateColumns,
    GridAutoRows,
    GridAutoColumns,
    GridRow,
    GridColumn,
    BackgroundColor,
    BorderColor,
    FocusPolicy,
    Visibility,
    ZIndex,
    Image,
    ImageScaleMode,
    TextureAtlas,
    Material,
}

#[derive(Debug, Default)]
pub struct AttributeStateStyle<T> {
    _default: T,
    _hover: Option<T>,
    _pressed: Option<T>,
    _focused: Option<T>,
    _selected: Option<T>,
    _checked: Option<T>,
    _active: Option<T>,
    _disabled: Option<T>,
}

#[derive(Debug)]
pub enum AttributeStyle<T: Lerp, C: Component + ComponentController<TargetType = T>> {
    Static(T),
    PerState(AttributeStateStyle<T>),
    // TODO: Handle delay: should be on the status change level to also apply to PerState
    Animated(AnimatedInteraction<C>),
}

#[derive(Debug)]
pub enum StyledAttribute {
    BackgroundColor(AttributeStyle<Color, InteractiveBackground>),
    Custom(fn(Entity, &mut World)),
}

impl PartialEq for StyledAttribute {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::BackgroundColor(_), Self::BackgroundColor(_)) => true,
            (Self::Custom(l0), Self::Custom(r0)) => l0 == r0,
            _ => false,
        }
    }
}

#[derive(Component, Debug, Default)]
pub struct LockedStyleAttributes(HashSet<StylableAttribute>);

impl LockedStyleAttributes {
    pub fn contains(&self, attr: StylableAttribute) -> bool {
        self.0.contains(&attr)
    }
}

#[derive(Component, Debug)]
pub struct Theme<C>
where
    C: Component,
{
    context: PhantomData<C>,
    style: Vec<StyledAttribute>,
}

impl<C> Theme<C>
where
    C: Component,
{
    pub fn add_style(&mut self, attribute: StyledAttribute) {
        if !self.style.contains(&attribute) {
            self.style.push(attribute);
        }
    }

    pub fn remove_style(&mut self, attribute: StyledAttribute) {
        if let Some(index) = self.style.iter().position(|e| e == &attribute) {
            self.style.remove(index);
        }
    }
}

// fn apply_theme<C>(
//     q_added_targets: Query<Entity, Added<C>>,
//     q_target: Query<(Entity, &C)>,
//     q_theme: Query<&Theme<C>>,
//     q_changed_theme: Query<(Entity, &Theme<C>), Changed<Theme<C>>>,
//     q_parent: Query<&Parent>,
//     mut commands: Commands,
// ) where
//     C: Component,
// {
// }

// fn follow_theme<C>(
//     q_changed_theme: Query<(Entity, &Theme<C>), Changed<Theme<C>>>,
//     q_target: Query<(Entity, &C)>,
//     q_parent: Query<&Parent>,
// ) where
//     C: Component,
// {
// }
