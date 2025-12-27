use bevy::prelude::*;
use crate::player::config::PlayerStateType;

/// Marker component for the player entity
#[derive(Component)]
pub struct Player;

/// Jump physics component - handles vertical movement and jump state
#[derive(Component)]
pub struct JumpPhysics {
    pub velocity_y: f32,
    pub ground_y: f32,
    pub jump_force: f32,
    /// Tracks whether player has used aerial attack during current jump
    /// Resets when landing or starting new jump
    pub has_used_aerial_attack: bool,
}

/// Combo window component - tracks attack chaining timing
#[derive(Component)]
pub struct ComboWindow {
    pub timer: Timer,
    pub last_attack: Option<PlayerStateType>,
    /// Queued combo state - will transition after current animation finishes
    pub queued_combo: Option<PlayerStateType>,
}
