use crate::player::config::*;
use crate::player::state::StateLogic;

/// Punch state - basic punch attack
#[derive(Clone, Default, Debug)]
pub struct PunchStateData;

impl StateLogic for PunchStateData {
    fn handle_input(&self, input: &InputContext) -> StateTransition {
        // Check for combo input during second half of attack animation
        if input.up_arrow {
            // Only allow combo queueing in second half of animation
            if input.current_frame >= input.total_frames / 2 {
                return StateTransition::QueueCombo(PlayerStateType::PunchCombo);
            }
        }

        // Input locked during attack animation
        StateTransition::None
    }

    fn update(&self, ctx: &UpdateContext) -> StateTransition {
        // When animation completes, check for queued combo or return to idle
        if ctx.animation_finished {
            // Queued combo will be checked by system layer
            return StateTransition::To(PlayerStateType::Idle);
        }

        StateTransition::None
    }

    fn get_animation_config(&self) -> AnimationConfig {
        AnimationConfig {
            sprite_path: "player/punch-sheet.png",
            first_frame: 1,
            last_frame: 12, // 13 frames total
            frame_duration: 0.03,
        }
    }

    fn get_physics_config(&self) -> PhysicsConfig {
        PhysicsConfig {
            ground_speed: 0.0,
            air_control: false,
            apply_gravity: false,
            locks_movement: true, // Cannot move during attack
        }
    }

    fn is_attacking(&self) -> bool {
        true
    }

    fn get_damage(&self) -> i32 {
        2
    }
}

/// PunchCombo state - second punch in combo sequence
#[derive(Clone, Default, Debug)]
pub struct PunchComboStateData;

impl StateLogic for PunchComboStateData {
    fn handle_input(&self, input: &InputContext) -> StateTransition {
        // Can chain into kick combo during second half (punch combo → down arrow)
        if input.down_arrow {
            // Only allow combo queueing in second half of animation
            if input.current_frame >= input.total_frames / 2 {
                return StateTransition::QueueCombo(PlayerStateType::PunchKickCombo);
            }
        }

        // Input locked during attack animation
        StateTransition::None
    }

    fn update(&self, ctx: &UpdateContext) -> StateTransition {
        // Return to idle when animation completes
        if ctx.animation_finished {
            return StateTransition::To(PlayerStateType::Idle);
        }

        StateTransition::None
    }

    fn get_animation_config(&self) -> AnimationConfig {
        AnimationConfig {
            sprite_path: "player/punch-combo-sheet.png",
            first_frame: 1,
            last_frame: 7, // 8 frames total
            frame_duration: 0.05,
        }
    }

    fn get_physics_config(&self) -> PhysicsConfig {
        PhysicsConfig {
            ground_speed: 0.0,
            air_control: false,
            apply_gravity: false,
            locks_movement: true,
        }
    }

    fn is_attacking(&self) -> bool {
        true
    }

    fn get_damage(&self) -> i32 {
        2
    }
}
