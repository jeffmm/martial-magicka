use crate::player::config::*;
use crate::player::state::StateLogic;

/// IdleToWalk transition state
#[derive(Clone, Default, Debug)]
pub struct IdleToWalkStateData;

impl StateLogic for IdleToWalkStateData {
    fn handle_input(&self, input: &InputContext) -> StateTransition {
        // Attack inputs can interrupt transition
        if input.up_arrow {
            return StateTransition::To(PlayerStateType::Punch);
        }
        if input.down_arrow {
            return StateTransition::To(PlayerStateType::Kick);
        }

        // Jump input
        if input.space {
            return StateTransition::To(PlayerStateType::Jump);
        }

        // If no movement keys pressed, return to idle
        if !input.left && !input.right {
            return StateTransition::To(PlayerStateType::Idle);
        }

        StateTransition::None
    }

    fn update(&self, ctx: &UpdateContext) -> StateTransition {
        // Transition to Walk when animation completes
        if ctx.animation_finished {
            return StateTransition::To(PlayerStateType::Walk);
        }
        StateTransition::None
    }

    fn get_animation_config(&self) -> AnimationConfig {
        AnimationConfig {
            sprite_path: "player/idle-to-walk-sheet.png",
            first_frame: 1,
            last_frame: 6, // 7 frames total
            frame_duration: 0.06,
        }
    }

    fn get_physics_config(&self) -> PhysicsConfig {
        PhysicsConfig {
            ground_speed: 200.0, // Same as Walk
            air_control: false,
            apply_gravity: false,
            locks_movement: false,
        }
    }
}

/// IdleToRun transition state
#[derive(Clone, Default, Debug)]
pub struct IdleToRunStateData;

impl StateLogic for IdleToRunStateData {
    fn handle_input(&self, input: &InputContext) -> StateTransition {
        // Attack inputs can interrupt transition
        if input.up_arrow {
            return StateTransition::To(PlayerStateType::Punch);
        }
        if input.down_arrow {
            return StateTransition::To(PlayerStateType::Kick);
        }

        // Jump input
        if input.space {
            return StateTransition::To(PlayerStateType::Jump);
        }

        // If no movement keys pressed, return to idle
        if !input.left && !input.right {
            return StateTransition::To(PlayerStateType::Idle);
        }

        StateTransition::None
    }

    fn update(&self, ctx: &UpdateContext) -> StateTransition {
        // Transition to Run when animation completes
        if ctx.animation_finished {
            return StateTransition::To(PlayerStateType::Run);
        }
        StateTransition::None
    }

    fn get_animation_config(&self) -> AnimationConfig {
        AnimationConfig {
            sprite_path: "player/idle-to-run-sheet.png",
            first_frame: 1,
            last_frame: 7, // 8 frames total
            frame_duration: 0.06,
        }
    }

    fn get_physics_config(&self) -> PhysicsConfig {
        PhysicsConfig {
            ground_speed: 400.0, // Same as Run
            air_control: false,
            apply_gravity: false,
            locks_movement: false,
        }
    }
}

/// Walk state - slower movement
#[derive(Clone, Default, Debug)]
pub struct WalkStateData;

impl StateLogic for WalkStateData {
    fn handle_input(&self, input: &InputContext) -> StateTransition {
        // Attack inputs
        if input.up_arrow {
            return StateTransition::To(PlayerStateType::Punch);
        }
        if input.down_arrow {
            return StateTransition::To(PlayerStateType::Kick);
        }

        // Jump input
        if input.space {
            return StateTransition::To(PlayerStateType::Jump);
        }

        // If no movement keys pressed, return to idle
        if !input.left && !input.right {
            return StateTransition::To(PlayerStateType::Idle);
        }

        StateTransition::None
    }

    fn update(&self, _ctx: &UpdateContext) -> StateTransition {
        // Walk doesn't auto-transition
        StateTransition::None
    }

    fn get_animation_config(&self) -> AnimationConfig {
        AnimationConfig {
            sprite_path: "player/walk-sheet.png",
            first_frame: 1,
            last_frame: 11, // 12 frames total
            frame_duration: 0.09,
        }
    }

    fn get_physics_config(&self) -> PhysicsConfig {
        PhysicsConfig {
            ground_speed: 200.0,
            air_control: false,
            apply_gravity: false,
            locks_movement: false,
        }
    }
}

/// Run state - faster movement
#[derive(Clone, Default, Debug)]
pub struct RunStateData;

impl StateLogic for RunStateData {
    fn handle_input(&self, input: &InputContext) -> StateTransition {
        // Attack inputs
        if input.up_arrow {
            return StateTransition::To(PlayerStateType::Punch);
        }
        if input.down_arrow {
            return StateTransition::To(PlayerStateType::Kick);
        }

        // Jump input
        if input.space {
            return StateTransition::To(PlayerStateType::Jump);
        }

        // If no movement keys pressed, return to idle
        if !input.left && !input.right {
            return StateTransition::To(PlayerStateType::Idle);
        }

        StateTransition::None
    }

    fn update(&self, _ctx: &UpdateContext) -> StateTransition {
        // Run doesn't auto-transition
        StateTransition::None
    }

    fn get_animation_config(&self) -> AnimationConfig {
        AnimationConfig {
            sprite_path: "player/run-sheet.png",
            first_frame: 1,
            last_frame: 7, // 8 frames total
            frame_duration: 0.07,
        }
    }

    fn get_physics_config(&self) -> PhysicsConfig {
        PhysicsConfig {
            ground_speed: 600.0,
            air_control: false,
            apply_gravity: false,
            locks_movement: false,
        }
    }
}
