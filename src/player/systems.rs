use crate::common::{AnimationIndices, AnimationTimer, Direction};
use crate::player::components::{ComboWindow, JumpPhysics, Player};
use crate::player::config::{
    InputContext, StateTransition as PlayerStateTransition, UpdateContext,
};
use crate::player::state::PlayerState;
use bevy::prelude::*;

// Type aliases to simplify complex query types
type PlayerInputQuery<'a> = (
    &'a mut PlayerState,
    &'a JumpPhysics,
    &'a mut ComboWindow,
    &'a Sprite,
    &'a AnimationIndices,
);

type PlayerStateUpdateQuery<'a> = (
    &'a mut PlayerState,
    &'a AnimationTimer,
    &'a AnimationIndices,
    &'a Sprite,
    &'a mut ComboWindow,
    &'a mut JumpPhysics,
    &'a Transform,
);

type JumpPhysicsInitQuery<'a> = (&'a PlayerState, &'a Transform, &'a mut JumpPhysics);

type HitTrackingQuery<'a> = (
    &'a PlayerState,
    &'a mut crate::combat::components::HitTracking,
);

type SpriteUpdateQuery<'a> = (
    &'a PlayerState,
    &'a mut Sprite,
    &'a mut AnimationIndices,
    &'a mut AnimationTimer,
);

/// Phase 1: Handle player input and request state transitions
///
/// This system builds an InputContext from keyboard state and delegates to the
/// current state's handle_input method to determine transitions.
pub fn player_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<PlayerInputQuery<'static>, With<Player>>,
    time: Res<Time>,
    game_state: Res<crate::GameState>,
) {
    // Freeze input when game is over
    if game_state.game_over {
        return;
    }

    let Ok((mut state, jump_physics, mut combo_window, sprite, indices)) =
        player_query.single_mut()
    else {
        return;
    };

    // Tick combo window timer
    combo_window.timer.tick(time.delta());

    // Get current animation frame
    let current_frame = if let Some(atlas) = &sprite.texture_atlas {
        atlas.index
    } else {
        0
    };
    let total_frames = indices.last + 1;

    // Build input context from keyboard state
    let input = InputContext {
        left: keyboard.pressed(KeyCode::KeyA),
        right: keyboard.pressed(KeyCode::KeyD),
        shift: keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight),
        space: keyboard.just_pressed(KeyCode::Space),
        up_arrow: keyboard.just_pressed(KeyCode::ArrowUp),
        down_arrow: keyboard.just_pressed(KeyCode::ArrowDown),
        has_used_aerial_attack: jump_physics.has_used_aerial_attack,
        current_frame,
        total_frames,
    };

    // Check if input is locked (attack animations)
    if state.locks_input() {
        // Even with locked input, check for combo inputs (they bypass lock)
        if input.up_arrow || input.down_arrow {
            // Only allow combo if combo window is active
            if combo_window.timer.is_finished() {
                return; // Combo window expired, ignore input
            }
            // Fall through to state input handler for combo logic
        } else {
            return; // Input locked, ignore non-combo inputs
        }
    }

    // Delegate to state's input handler
    match state.handle_input(&input) {
        PlayerStateTransition::To(new_state_type) => {
            // Apply immediate state transition
            let new_state = PlayerState::transition_to(new_state_type);

            // Update combo window for attack states
            if new_state.is_attacking() {
                combo_window.last_attack = Some(new_state_type);
                combo_window.timer.reset();
            }

            *state = new_state;
        }
        PlayerStateTransition::QueueCombo(combo_state_type) => {
            // Queue combo to execute when current animation finishes
            // Only queue if combo window is still active
            if !combo_window.timer.is_finished() {
                combo_window.queued_combo = Some(combo_state_type);
            }
        }
        PlayerStateTransition::None => {
            // No transition
        }
    }
}

/// Phase 2: Update state based on animation/physics conditions
///
/// This system builds an UpdateContext from animation/physics state and delegates
/// to the current state's update method to determine automatic transitions.
pub fn player_state_update_system(
    mut player_query: Query<PlayerStateUpdateQuery<'static>, With<Player>>,
) {
    for (mut state, timer, indices, sprite, mut combo_window, jump_physics, transform) in
        player_query.iter_mut()
    {
        // Check if animation completed (looped back to first frame)
        let animation_finished = if let Some(atlas) = &sprite.texture_atlas {
            atlas.index == indices.last && timer.just_finished()
        } else {
            false
        };

        // Build update context
        let ctx = UpdateContext {
            animation_finished,
            is_at_ground: transform.translation.y <= jump_physics.ground_y + 1.0,
            velocity_y: jump_physics.velocity_y,
        };

        // Delegate to state's update handler
        if let PlayerStateTransition::To(new_state_type) = state.update(&ctx) {
            // Check if there's a queued combo that should override this transition
            if animation_finished && combo_window.queued_combo.is_some() {
                // Execute queued combo instead of normal transition
                let combo_state_type = combo_window.queued_combo.take().unwrap();
                *state = PlayerState::transition_to(combo_state_type);

                // Update combo window for the new attack
                combo_window.last_attack = Some(combo_state_type);
                combo_window.timer.reset();
            } else {
                // Apply normal state transition
                *state = PlayerState::transition_to(new_state_type);
            }
        }
    }
}

/// Separate system to initialize jump physics when transitioning to Jump state
/// and reset velocity when entering Fall state from aerial attacks.
/// This runs before player_state_update_system
pub fn initialize_jump_physics(
    mut player_query: Query<JumpPhysicsInitQuery<'static>, (With<Player>, Changed<PlayerState>)>,
) {
    for (state, transform, mut jump_physics) in player_query.iter_mut() {
        match state {
            PlayerState::Jump(_) => {
                // Store ground position and initialize jump velocity
                jump_physics.ground_y = transform.translation.y;
                jump_physics.velocity_y = jump_physics.jump_force;
                // Reset aerial attack flag - new jump allows one attack
                jump_physics.has_used_aerial_attack = false;
            }
            PlayerState::Fall(_) => {
                // Reset velocity to 0 when entering Fall state
                // This ensures aerial attacks cause immediate descent instead of
                // continuing upward with preserved jump velocity
                jump_physics.velocity_y = 0.0;
            }
            PlayerState::JumpPunch(_) | PlayerState::JumpKick(_) => {
                // Mark that player has used their aerial attack
                jump_physics.has_used_aerial_attack = true;
            }
            _ => {}
        }
    }
}

/// Clear hit tracking when entering a new attack state
/// This allows each attack to hit enemies independently
pub fn clear_hit_tracking_on_state_change(
    mut player_query: Query<HitTrackingQuery<'static>, (With<Player>, Changed<PlayerState>)>,
) {
    for (state, mut hit_tracking) in player_query.iter_mut() {
        // Clear hit tracking when entering any attack state
        if state.is_attacking() {
            hit_tracking.hit_enemies.clear();
        }
    }
}

/// Phase 3: Apply sprite changes when state changes
///
/// Uses preloaded sprite sheet handles to prevent flickering during transitions.
/// The critical ordering ensures sprites remain visible throughout state changes.
pub fn player_sprite_update_system(
    mut player_query: Query<SpriteUpdateQuery<'static>, (With<Player>, Changed<PlayerState>)>,
    sprite_sheets: Res<crate::PlayerSpriteSheets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    for (state, mut sprite, mut indices, mut timer) in player_query.iter_mut() {
        // Get animation config from state
        let anim = state.get_animation_config();
        let num_columns = (anim.last_frame + 1) as u32;

        // Use preloaded sprite handle instead of loading on-demand
        // This prevents flickering since the texture is already in GPU memory
        sprite.image = sprite_sheets.get_handle(anim.sprite_path);

        // Update texture atlas
        if let Some(ref mut atlas) = sprite.texture_atlas {
            // Reset to first frame BEFORE changing layout to prevent out-of-bounds access
            atlas.index = anim.first_frame;

            // Create new texture atlas layout with correct number of columns
            atlas.layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
                UVec2::splat(320),
                num_columns,
                1,
                None,
                None,
            ));
        }

        // Update animation indices
        indices.first = anim.first_frame;
        indices.last = anim.last_frame;

        // Set timer duration based on animation state and reset
        timer.set_duration(std::time::Duration::from_secs_f32(anim.frame_duration));
        timer.reset();
    }
}

/// Phase 4: Apply physics based on state configuration
///
/// This system reads the physics config from the current state and applies
/// gravity, air control, and ground movement accordingly.
pub fn player_physics_system(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<
        (
            &PlayerState,
            &mut Transform,
            &mut Sprite,
            &mut JumpPhysics,
            &mut Direction,
        ),
        With<Player>,
    >,
) {
    const GRAVITY: f32 = 1800.0;
    const AIR_CONTROL_SPEED: f32 = 250.0;

    for (state, mut transform, mut sprite, mut jump_physics, mut direction) in
        player_query.iter_mut()
    {
        let physics_config = state.get_physics_config();

        // Apply gravity if needed
        if physics_config.apply_gravity {
            jump_physics.velocity_y -= GRAVITY * time.delta_secs();
            transform.translation.y += jump_physics.velocity_y * time.delta_secs();

            // Ground clamp
            if transform.translation.y < jump_physics.ground_y {
                transform.translation.y = jump_physics.ground_y;
                jump_physics.velocity_y = 0.0;
            }
        }

        // Apply air control if allowed
        if physics_config.air_control {
            if keyboard.pressed(KeyCode::KeyA) {
                transform.translation.x -= AIR_CONTROL_SPEED * time.delta_secs();
                sprite.flip_x = true;
                *direction = Direction::Left;
            } else if keyboard.pressed(KeyCode::KeyD) {
                transform.translation.x += AIR_CONTROL_SPEED * time.delta_secs();
                sprite.flip_x = false;
                *direction = Direction::Right;
            }
        }

        // Apply ground movement if not locked and speed > 0
        if !physics_config.locks_movement && physics_config.ground_speed > 0.0 {
            if keyboard.pressed(KeyCode::KeyA) {
                transform.translation.x -= physics_config.ground_speed * time.delta_secs();
                sprite.flip_x = true;
                *direction = Direction::Left;
            } else if keyboard.pressed(KeyCode::KeyD) {
                transform.translation.x += physics_config.ground_speed * time.delta_secs();
                sprite.flip_x = false;
                *direction = Direction::Right;
            }
        }

        // Always enforce ground clamping (even when gravity is not active)
        // This prevents knockback or other forces from pushing player below ground
        if transform.translation.y < jump_physics.ground_y {
            transform.translation.y = jump_physics.ground_y;
            jump_physics.velocity_y = 0.0;
        }
    }
}
