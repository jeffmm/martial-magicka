use bevy::prelude::*;

/// Message sent when damage is dealt
#[derive(Message)]
pub struct DamageEvent {
    pub attacker: Entity,
    pub target: Entity,
    pub damage: i32,
}

/// Message sent when an enemy is defeated
#[derive(Message)]
pub struct EnemyDefeatedEvent {
    pub enemy: Entity,
}

/// Message sent when the player is defeated
#[derive(Message)]
pub struct PlayerDefeatedEvent;
