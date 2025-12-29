// No bevy imports needed for config types

/// Animation configuration for a player state
#[derive(Clone, Debug)]
pub struct AnimationConfig {
    pub sprite_path: &'static str,
    pub first_frame: usize,
    pub last_frame: usize,
    pub frame_duration: f32,
}

/// Physics behavior configuration for a player state
#[derive(Clone, Debug, Default)]
pub struct PhysicsConfig {
    /// Speed when moving on the ground (0.0 for non-moving states)
    pub ground_speed: f32,
    /// Whether the player can steer in the air with A/D
    pub air_control: bool,
    /// Should gravity be applied in this state
    pub apply_gravity: bool,
    /// Cannot move during this state (e.g., attacks)
    pub locks_movement: bool,
}

/// Input context passed to state's handle_input method
#[derive(Clone, Debug, Default)]
pub struct InputContext {
    pub left: bool,
    pub right: bool,
    pub shift: bool,
    pub space: bool,
    pub up_arrow: bool,
    pub down_arrow: bool,
    /// True if player has already used an aerial attack during this jump
    pub has_used_aerial_attack: bool,
    /// Current animation frame index
    pub current_frame: usize,
    /// Total frames in animation (last_frame + 1)
    pub total_frames: usize,
}

/// Update context passed to state's update method
#[derive(Clone, Debug)]
pub struct UpdateContext {
    /// True if the animation just completed (looped back to first frame)
    pub animation_finished: bool,
    /// True if the player is at or below ground level
    pub is_at_ground: bool,
    /// Current vertical velocity
    pub velocity_y: f32,
}

/// Lightweight enum representing state types (for transitions)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayerStateType {
    Idle,
    IdleToWalk,
    IdleToRun,
    Walk,
    Run,
    Jump,
    Fall,
    Land,
    Punch,
    PunchCombo,
    Kick,
    KickCombo,
    PunchKickCombo,
    JumpPunch,
    JumpKick,
    Defeat,
}

/// State transition result
#[derive(Clone, Debug)]
pub enum StateTransition {
    /// Transition to a new state immediately
    To(PlayerStateType),
    /// Queue a combo state to transition to when current animation finishes
    QueueCombo(PlayerStateType),
    /// No transition, stay in current state
    None,
}
