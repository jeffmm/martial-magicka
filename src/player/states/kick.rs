use crate::player::config::*;
use crate::player::state::StateLogic;

/// Kick state - basic kick attack
#[derive(Clone, Default, Debug)]
pub struct KickStateData;

impl StateLogic for KickStateData {
    fn handle_input(&self, input: &InputContext) -> StateTransition {
        // Check for combo input during second half of attack animation
        if input.down_arrow {
            // Only allow combo queueing in second half of animation
            if input.current_frame >= input.total_frames / 2 {
                return StateTransition::QueueCombo(PlayerStateType::KickCombo);
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
            sprite_path: "player/kick-sheet.png",
            first_frame: 1,
            last_frame: 20, // 21 frames total
            frame_duration: 0.02,
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
        3 // Kicks do more damage than punches
    }
}

/// KickCombo state - second kick in combo sequence
#[derive(Clone, Default, Debug)]
pub struct KickComboStateData;

impl StateLogic for KickComboStateData {
    fn handle_input(&self, _input: &InputContext) -> StateTransition {
        // No further combos from kick combo
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
            sprite_path: "player/kick-combo-sheet.png",
            first_frame: 1,
            last_frame: 19, // 20 frames total
            frame_duration: 0.02,
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
        3
    }
}
