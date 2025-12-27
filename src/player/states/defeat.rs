use crate::player::config::*;
use crate::player::state::StateLogic;

/// Defeat state - player has been defeated
/// Plays death animation once and freezes on last frame
#[derive(Clone, Default, Debug)]
pub struct DefeatStateData;

impl StateLogic for DefeatStateData {
    fn handle_input(&self, _input: &InputContext) -> StateTransition {
        // No input accepted in defeat state
        StateTransition::None
    }

    fn update(&self, _ctx: &UpdateContext) -> StateTransition {
        // Stay in defeat state - don't transition even when animation finishes
        // The animation system will loop, but we freeze on last frame via sprite update
        StateTransition::None
    }

    fn get_animation_config(&self) -> AnimationConfig {
        AnimationConfig {
            sprite_path: "player/defeat-sheet.png",
            first_frame: 1,
            last_frame: 20, // 21 frames total (6720px / 320px)
            frame_duration: 0.1,
        }
    }

    fn get_physics_config(&self) -> PhysicsConfig {
        PhysicsConfig {
            ground_speed: 0.0,
            air_control: false,
            apply_gravity: false,
            locks_movement: true, // Cannot move when defeated
        }
    }
}
