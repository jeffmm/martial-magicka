use bevy::prelude::*;

/// Marker component for enemy entities
#[derive(Component)]
pub struct Enemy;

/// Enemy AI state
#[derive(Component)]
pub enum EnemyState {
    Idle,
    Move,
}
