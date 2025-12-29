use super::config::*;
use super::states::*;
use bevy::prelude::Component;

/// Trait that all player states must implement
/// Defines the behavior for input handling, state updates, animation, and physics
pub trait StateLogic: Send + Sync + Clone {
    /// Handle keyboard input and return a state transition if applicable
    ///
    /// This is called during the input handling phase and allows the state
    /// to respond to player actions (movement, jumping, attacks, etc.)
    fn handle_input(&self, input: &InputContext) -> StateTransition;

    /// Update the state based on animation/physics conditions
    ///
    /// This is called during the state update phase and allows the state to
    /// transition based on animation completion, physics conditions, etc.
    fn update(&self, ctx: &UpdateContext) -> StateTransition;

    /// Get the animation configuration for this state
    ///
    /// Returns sprite path, frame range, and timing information
    fn get_animation_config(&self) -> AnimationConfig;

    /// Get the physics behavior configuration for this state
    ///
    /// Returns movement speed, gravity settings, and input lock status
    fn get_physics_config(&self) -> PhysicsConfig;

    /// Is this an attacking state?
    ///
    /// Default: false. Override in attack states (Punch, Kick, etc.)
    fn is_attacking(&self) -> bool {
        false
    }

    /// Get the damage dealt by this state
    ///
    /// Default: 0. Override in attack states to return damage amount
    fn get_damage(&self) -> i32 {
        0
    }
}

/// Player state component - each variant holds a state behavior object
#[derive(Component, Clone)]
pub enum PlayerState {
    Idle(IdleStateData),
    IdleToWalk(IdleToWalkStateData),
    IdleToRun(IdleToRunStateData),
    Walk(WalkStateData),
    Run(RunStateData),
    Jump(JumpStateData),
    Fall(FallStateData),
    Land(LandStateData),
    Punch(PunchStateData),
    PunchCombo(PunchComboStateData),
    Kick(KickStateData),
    KickCombo(KickComboStateData),
    PunchKickCombo(PunchKickComboStateData),
    JumpPunch(JumpPunchStateData),
    JumpKick(JumpKickStateData),
    Defeat(DefeatStateData),
}

impl PlayerState {
    /// Delegate to embedded state's input handler
    pub fn handle_input(&self, input: &InputContext) -> StateTransition {
        match self {
            PlayerState::Idle(s) => s.handle_input(input),
            PlayerState::IdleToWalk(s) => s.handle_input(input),
            PlayerState::IdleToRun(s) => s.handle_input(input),
            PlayerState::Walk(s) => s.handle_input(input),
            PlayerState::Run(s) => s.handle_input(input),
            PlayerState::Jump(s) => s.handle_input(input),
            PlayerState::Fall(s) => s.handle_input(input),
            PlayerState::Land(s) => s.handle_input(input),
            PlayerState::Punch(s) => s.handle_input(input),
            PlayerState::PunchCombo(s) => s.handle_input(input),
            PlayerState::Kick(s) => s.handle_input(input),
            PlayerState::KickCombo(s) => s.handle_input(input),
            PlayerState::PunchKickCombo(s) => s.handle_input(input),
            PlayerState::JumpPunch(s) => s.handle_input(input),
            PlayerState::JumpKick(s) => s.handle_input(input),
            PlayerState::Defeat(s) => s.handle_input(input),
        }
    }

    /// Delegate to embedded state's update method
    pub fn update(&self, ctx: &UpdateContext) -> StateTransition {
        match self {
            PlayerState::Idle(s) => s.update(ctx),
            PlayerState::IdleToWalk(s) => s.update(ctx),
            PlayerState::IdleToRun(s) => s.update(ctx),
            PlayerState::Walk(s) => s.update(ctx),
            PlayerState::Run(s) => s.update(ctx),
            PlayerState::Jump(s) => s.update(ctx),
            PlayerState::Fall(s) => s.update(ctx),
            PlayerState::Land(s) => s.update(ctx),
            PlayerState::Punch(s) => s.update(ctx),
            PlayerState::PunchCombo(s) => s.update(ctx),
            PlayerState::Kick(s) => s.update(ctx),
            PlayerState::KickCombo(s) => s.update(ctx),
            PlayerState::PunchKickCombo(s) => s.update(ctx),
            PlayerState::JumpPunch(s) => s.update(ctx),
            PlayerState::JumpKick(s) => s.update(ctx),
            PlayerState::Defeat(s) => s.update(ctx),
        }
    }

    /// Delegate to embedded state's animation config
    pub fn get_animation_config(&self) -> AnimationConfig {
        match self {
            PlayerState::Idle(s) => s.get_animation_config(),
            PlayerState::IdleToWalk(s) => s.get_animation_config(),
            PlayerState::IdleToRun(s) => s.get_animation_config(),
            PlayerState::Walk(s) => s.get_animation_config(),
            PlayerState::Run(s) => s.get_animation_config(),
            PlayerState::Jump(s) => s.get_animation_config(),
            PlayerState::Fall(s) => s.get_animation_config(),
            PlayerState::Land(s) => s.get_animation_config(),
            PlayerState::Punch(s) => s.get_animation_config(),
            PlayerState::PunchCombo(s) => s.get_animation_config(),
            PlayerState::Kick(s) => s.get_animation_config(),
            PlayerState::KickCombo(s) => s.get_animation_config(),
            PlayerState::PunchKickCombo(s) => s.get_animation_config(),
            PlayerState::JumpPunch(s) => s.get_animation_config(),
            PlayerState::JumpKick(s) => s.get_animation_config(),
            PlayerState::Defeat(s) => s.get_animation_config(),
        }
    }

    /// Delegate to embedded state's physics config
    pub fn get_physics_config(&self) -> PhysicsConfig {
        match self {
            PlayerState::Idle(s) => s.get_physics_config(),
            PlayerState::IdleToWalk(s) => s.get_physics_config(),
            PlayerState::IdleToRun(s) => s.get_physics_config(),
            PlayerState::Walk(s) => s.get_physics_config(),
            PlayerState::Run(s) => s.get_physics_config(),
            PlayerState::Jump(s) => s.get_physics_config(),
            PlayerState::Fall(s) => s.get_physics_config(),
            PlayerState::Land(s) => s.get_physics_config(),
            PlayerState::Punch(s) => s.get_physics_config(),
            PlayerState::PunchCombo(s) => s.get_physics_config(),
            PlayerState::Kick(s) => s.get_physics_config(),
            PlayerState::KickCombo(s) => s.get_physics_config(),
            PlayerState::PunchKickCombo(s) => s.get_physics_config(),
            PlayerState::JumpPunch(s) => s.get_physics_config(),
            PlayerState::JumpKick(s) => s.get_physics_config(),
            PlayerState::Defeat(s) => s.get_physics_config(),
        }
    }

    /// Does this state lock player input?
    pub fn locks_input(&self) -> bool {
        self.get_physics_config().locks_movement
    }

    /// Is this an attacking state?
    pub fn is_attacking(&self) -> bool {
        match self {
            PlayerState::Idle(s) => s.is_attacking(),
            PlayerState::IdleToWalk(s) => s.is_attacking(),
            PlayerState::IdleToRun(s) => s.is_attacking(),
            PlayerState::Walk(s) => s.is_attacking(),
            PlayerState::Run(s) => s.is_attacking(),
            PlayerState::Jump(s) => s.is_attacking(),
            PlayerState::Fall(s) => s.is_attacking(),
            PlayerState::Land(s) => s.is_attacking(),
            PlayerState::Punch(s) => s.is_attacking(),
            PlayerState::PunchCombo(s) => s.is_attacking(),
            PlayerState::Kick(s) => s.is_attacking(),
            PlayerState::KickCombo(s) => s.is_attacking(),
            PlayerState::PunchKickCombo(s) => s.is_attacking(),
            PlayerState::JumpPunch(s) => s.is_attacking(),
            PlayerState::JumpKick(s) => s.is_attacking(),
            PlayerState::Defeat(s) => s.is_attacking(),
        }
    }

    /// Get damage dealt by this state
    pub fn get_damage(&self) -> i32 {
        match self {
            PlayerState::Idle(s) => s.get_damage(),
            PlayerState::IdleToWalk(s) => s.get_damage(),
            PlayerState::IdleToRun(s) => s.get_damage(),
            PlayerState::Walk(s) => s.get_damage(),
            PlayerState::Run(s) => s.get_damage(),
            PlayerState::Jump(s) => s.get_damage(),
            PlayerState::Fall(s) => s.get_damage(),
            PlayerState::Land(s) => s.get_damage(),
            PlayerState::Punch(s) => s.get_damage(),
            PlayerState::PunchCombo(s) => s.get_damage(),
            PlayerState::Kick(s) => s.get_damage(),
            PlayerState::KickCombo(s) => s.get_damage(),
            PlayerState::PunchKickCombo(s) => s.get_damage(),
            PlayerState::JumpPunch(s) => s.get_damage(),
            PlayerState::JumpKick(s) => s.get_damage(),
            PlayerState::Defeat(s) => s.get_damage(),
        }
    }

    /// Factory method to create new state instances
    pub fn transition_to(state_type: PlayerStateType) -> Self {
        match state_type {
            PlayerStateType::Idle => PlayerState::Idle(IdleStateData),
            PlayerStateType::IdleToWalk => PlayerState::IdleToWalk(IdleToWalkStateData),
            PlayerStateType::IdleToRun => PlayerState::IdleToRun(IdleToRunStateData),
            PlayerStateType::Walk => PlayerState::Walk(WalkStateData),
            PlayerStateType::Run => PlayerState::Run(RunStateData),
            PlayerStateType::Jump => PlayerState::Jump(JumpStateData),
            PlayerStateType::Fall => PlayerState::Fall(FallStateData),
            PlayerStateType::Land => PlayerState::Land(LandStateData),
            PlayerStateType::Punch => PlayerState::Punch(PunchStateData),
            PlayerStateType::PunchCombo => PlayerState::PunchCombo(PunchComboStateData),
            PlayerStateType::Kick => PlayerState::Kick(KickStateData),
            PlayerStateType::KickCombo => PlayerState::KickCombo(KickComboStateData),
            PlayerStateType::PunchKickCombo => PlayerState::PunchKickCombo(PunchKickComboStateData),
            PlayerStateType::JumpPunch => PlayerState::JumpPunch(JumpPunchStateData),
            PlayerStateType::JumpKick => PlayerState::JumpKick(JumpKickStateData),
            PlayerStateType::Defeat => PlayerState::Defeat(DefeatStateData),
        }
    }
}

// Implement PartialEq for state type comparison (needed for queries and pattern matching)
impl PartialEq for PlayerState {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

impl Eq for PlayerState {}
