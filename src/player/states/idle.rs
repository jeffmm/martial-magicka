use crate::player::config::*;
use crate::player::state::StateLogic;

/// Idle state - player is standing still
#[derive(Clone, Default, Debug)]
pub struct IdleStateData;

impl StateLogic for IdleStateData {
    fn handle_input(&self, input: &InputContext) -> StateTransition {
        // Attack inputs have highest priority
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

        // Movement inputs (lower priority)
        if input.left || input.right {
            if input.shift {
                return StateTransition::To(PlayerStateType::IdleToWalk);
            } else {
                return StateTransition::To(PlayerStateType::IdleToRun);
            }
        }

        // No input, stay idle
        StateTransition::None
    }

    fn update(&self, _ctx: &UpdateContext) -> StateTransition {
        // Idle doesn't auto-transition based on animation or physics
        StateTransition::None
    }

    fn get_animation_config(&self) -> AnimationConfig {
        AnimationConfig {
            sprite_path: "player/idle-sheet.png",
            first_frame: 1,
            last_frame: 23, // 24 frames total (7680px / 320px)
            frame_duration: 0.12,
        }
    }

    fn get_physics_config(&self) -> PhysicsConfig {
        PhysicsConfig {
            ground_speed: 0.0,
            air_control: false,
            apply_gravity: false,
            locks_movement: false,
        }
    }
}
