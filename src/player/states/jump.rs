use crate::player::config::*;
use crate::player::state::StateLogic;

/// Jump state - ascending phase of jump
#[derive(Clone, Default, Debug)]
pub struct JumpStateData;

impl StateLogic for JumpStateData {
    fn handle_input(&self, input: &InputContext) -> StateTransition {
        // Can attack mid-air (but only once per jump)
        if !input.has_used_aerial_attack {
            if input.up_arrow {
                return StateTransition::To(PlayerStateType::JumpPunch);
            }
            if input.down_arrow {
                return StateTransition::To(PlayerStateType::JumpKick);
            }
        }

        // A/D for air control is handled by physics system
        StateTransition::None
    }

    fn update(&self, ctx: &UpdateContext) -> StateTransition {
        // Transition to Fall when reaching peak (velocity becomes negative or zero)
        if ctx.velocity_y <= 0.0 {
            return StateTransition::To(PlayerStateType::Fall);
        }

        StateTransition::None
    }

    fn get_animation_config(&self) -> AnimationConfig {
        AnimationConfig {
            sprite_path: "player/jump-sheet.png",
            first_frame: 1,
            last_frame: 26, // 27 frames total (8640px / 320px)
            frame_duration: 0.05,
        }
    }

    fn get_physics_config(&self) -> PhysicsConfig {
        PhysicsConfig {
            ground_speed: 0.0,     // No ground movement
            air_control: true,     // Can steer with A/D
            apply_gravity: true,   // Gravity slows upward velocity
            locks_movement: false,
        }
    }
}

/// Fall state - descending phase of jump
#[derive(Clone, Default, Debug)]
pub struct FallStateData;

impl StateLogic for FallStateData {
    fn handle_input(&self, input: &InputContext) -> StateTransition {
        // Can attack while falling (but only once per jump)
        if !input.has_used_aerial_attack {
            if input.up_arrow {
                return StateTransition::To(PlayerStateType::JumpPunch);
            }
            if input.down_arrow {
                return StateTransition::To(PlayerStateType::JumpKick);
            }
        }

        // A/D for air control is handled by physics system
        StateTransition::None
    }

    fn update(&self, ctx: &UpdateContext) -> StateTransition {
        // Transition to Land when touching ground
        if ctx.is_at_ground {
            return StateTransition::To(PlayerStateType::Land);
        }

        StateTransition::None
    }

    fn get_animation_config(&self) -> AnimationConfig {
        AnimationConfig {
            sprite_path: "player/falling-sheet.png",
            first_frame: 1,
            last_frame: 19, // 20 frames total
            frame_duration: 0.1,
        }
    }

    fn get_physics_config(&self) -> PhysicsConfig {
        PhysicsConfig {
            ground_speed: 0.0,
            air_control: true,     // Can steer with A/D
            apply_gravity: true,   // Gravity accelerates downward
            locks_movement: false,
        }
    }
}

/// Land state - landing animation after jump
#[derive(Clone, Default, Debug)]
pub struct LandStateData;

impl StateLogic for LandStateData {
    fn handle_input(&self, _input: &InputContext) -> StateTransition {
        // Cannot interrupt landing animation with input
        StateTransition::None
    }

    fn update(&self, ctx: &UpdateContext) -> StateTransition {
        // Return to idle when landing animation completes
        if ctx.animation_finished {
            return StateTransition::To(PlayerStateType::Idle);
        }

        StateTransition::None
    }

    fn get_animation_config(&self) -> AnimationConfig {
        AnimationConfig {
            sprite_path: "player/landing-sheet.png",
            first_frame: 1,
            last_frame: 20, // 21 frames total
            frame_duration: 0.02,
        }
    }

    fn get_physics_config(&self) -> PhysicsConfig {
        PhysicsConfig {
            ground_speed: 0.0,
            air_control: false,
            apply_gravity: false, // Locked at ground level during landing
            locks_movement: true, // Cannot move during landing animation
        }
    }
}
