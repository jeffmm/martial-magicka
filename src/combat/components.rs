use bevy::prelude::*;
use std::collections::HashSet;

/// Health component for entities that can take damage
#[derive(Component)]
pub struct Health {
    pub current: i32,
    pub max: i32,
}

/// Hitbox for attack collision detection (attacker)
#[derive(Component)]
pub struct Hitbox {
    pub offset: Vec2,
    pub size: Vec2,
    pub active: bool,
}

/// HurtBox for receiving damage (defender)
#[derive(Component)]
pub struct HurtBox {
    pub size: Vec2,
}

/// Stunned component - entity cannot move when stunned
#[derive(Component)]
pub struct Stunned {
    pub timer: Timer,
}

/// Knockback velocity applied when an entity is hit
#[derive(Component)]
pub struct Knockback {
    pub velocity: Vec2,
}

/// Visual hit flash effect - entity glows red when hit
#[derive(Component)]
pub struct HitFlash {
    pub timer: Timer,
    pub flash_duration: f32,
}

/// Invulnerability frames - entity cannot take damage during this window
#[derive(Component)]
pub struct Invulnerable {
    pub timer: Timer,
}

/// Tracks which enemies have been hit by the current attack
/// Prevents the same attack from hitting an enemy multiple times
#[derive(Component, Default)]
pub struct HitTracking {
    pub hit_enemies: HashSet<Entity>,
}
