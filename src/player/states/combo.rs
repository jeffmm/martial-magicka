use crate::player::config::*;
use crate::player::state::StateLogic;

/// PunchKickCombo state - mixed combo (punch combo → kick)
/// This is the final attack in the punch-kick combo chain
#[derive(Clone, Default, Debug)]
pub struct PunchKickComboStateData;

impl StateLogic for PunchKickComboStateData {
    fn handle_input(&self, _input: &InputContext) -> StateTransition {
        // No further combos from punch-kick combo
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
            sprite_path: "player/punch-kick-combo-sheet.png",
            first_frame: 1,
            last_frame: 16, // 17 frames total
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
        3 // Kick damage (ends with a kick)
    }
}
