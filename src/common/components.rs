use bevy::prelude::*;

/// Direction the entity is facing or moving
#[derive(Component)]
pub enum Direction {
    None,
    Left,
    Right,
}

/// Animation frame indices (first and last frame)
#[derive(Component)]
pub struct AnimationIndices {
    pub first: usize,
    pub last: usize,
}

/// Timer for animation frame progression
#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(pub Timer);
