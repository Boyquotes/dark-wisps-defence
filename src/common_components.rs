use bevy::prelude::*;

#[derive(Component, Default)]
pub struct Health(pub i32);

#[derive(Component, Default)]
pub struct TargetVector(pub(crate) Vec2);