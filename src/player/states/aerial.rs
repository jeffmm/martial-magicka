use crate::player::config::*;
use crate::player::state::StateLogic;

/// JumpPunch state - aerial punch attack
#[derive(Clone, Default, Debug)]
pub struct JumpPunchStateData;

impl StateLogic for JumpPunchStateData {
    fn handle_input(&self, _input: &InputContext) -> StateTransition {
        // Input locked during aerial attack
        // Player can still control horizontal movement via physics system
        StateTransition::None
    }

    fn update(&self, ctx: &UpdateContext) -> StateTransition {
        // When animation completes, check if still airborne
        if ctx.animation_finished {
            if !ctx.is_at_ground {
                // Still in air, transition to falling
                return StateTransition::To(PlayerStateType::Fall);
            } else {
                // On ground, land immediately
                return StateTransition::To(PlayerStateType::Land);
            }
        }

        StateTransition::None
    }

    fn get_animation_config(&self) -> AnimationConfig {
        AnimationConfig {
            sprite_path: "player/jump-punch-sheet.png",
            first_frame: 1,
            last_frame: 17, // 18 frames total
            frame_duration: 0.02,
        }
    }

    fn get_physics_config(&self) -> PhysicsConfig {
        PhysicsConfig {
            ground_speed: 0.0,
            air_control: true,     // Can steer with A/D during attack
            apply_gravity: false,  // Freeze height during attack (arcade-style)
            locks_movement: false, // Air control is allowed
        }
    }

    fn is_attacking(&self) -> bool {
        true
    }

    fn get_damage(&self) -> i32 {
        6 // Aerial attacks are more powerful - one-shot ghosts
    }
}

/// JumpKick state - aerial kick attack
#[derive(Clone, Default, Debug)]
pub struct JumpKickStateData;

impl StateLogic for JumpKickStateData {
    fn handle_input(&self, _input: &InputContext) -> StateTransition {
        // Input locked during aerial attack
        // Player can still control horizontal movement via physics system
        StateTransition::None
    }

    fn update(&self, ctx: &UpdateContext) -> StateTransition {
        // When animation completes, check if still airborne
        if ctx.animation_finished {
            if !ctx.is_at_ground {
                // Still in air, transition to falling
                return StateTransition::To(PlayerStateType::Fall);
            } else {
                // On ground, land immediately
                return StateTransition::To(PlayerStateType::Land);
            }
        }

        StateTransition::None
    }

    fn get_animation_config(&self) -> AnimationConfig {
        AnimationConfig {
            sprite_path: "player/jump-kick-sheet.png",
            first_frame: 1,
            last_frame: 19, // 20 frames total
            frame_duration: 0.02,
        }
    }

    fn get_physics_config(&self) -> PhysicsConfig {
        PhysicsConfig {
            ground_speed: 0.0,
            air_control: true,     // Can steer with A/D during attack
            apply_gravity: false,  // Freeze height during attack (arcade-style)
            locks_movement: false, // Air control is allowed
        }
    }

    fn is_attacking(&self) -> bool {
        true
    }

    fn get_damage(&self) -> i32 {
        6 // Aerial attacks are more powerful - one-shot ghosts
    }
}
