mod combat;
mod common;
mod enemy;
mod player;

use bevy::prelude::*;
use combat::{
    DamageEvent, EnemyDefeatedEvent, Health, HitFlash, HitTracking, Hitbox, HurtBox, Invulnerable,
    Knockback, PlayerDefeatedEvent, Stunned,
};
use common::{AnimationIndices, AnimationTimer, Direction};
use enemy::{Enemy, EnemyState};
use player::systems::*;
use player::{ComboWindow, JumpPhysics, Player, PlayerState, PlayerStateType};

const MAX_ENEMIES: u32 = 6;

// UI Marker Components
#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct HealthText;

#[derive(Component)]
struct TimeText;

#[derive(Component)]
struct GameOverScreen;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_message::<DamageEvent>()
        .add_message::<EnemyDefeatedEvent>()
        .add_message::<PlayerDefeatedEvent>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                (
                    // Phase 1: Input & State Management (NEW MODULAR SYSTEMS)
                    player_input_system,
                    initialize_jump_physics,
                    clear_hit_tracking_on_state_change,
                    player_state_update_system,
                    player_sprite_update_system,
                    // Phase 2: Movement (NEW PHYSICS SYSTEM)
                    player_physics_system,
                    // Phase 3: Combat & Collision
                    update_attack_hitboxes,
                    detect_combat_collisions,
                    detect_player_enemy_collisions,
                    // Phase 4: Damage Resolution
                    handle_damage_events,
                    update_stun_timers,
                )
                    .chain(),
                (
                    update_invulnerability,
                    handle_enemy_defeat,
                    handle_player_defeat,
                    // Phase 5: Enemy AI & Knockback
                    move_enemies,
                    apply_knockback,
                    // Phase 6: Visual Effects & Game Management
                    update_hit_flash,
                    animate_sprite,
                    count_down,
                    spawn_enemy,
                    // Phase 7: UI Updates
                    update_ui,
                    handle_game_over,
                    handle_restart,
                )
                    .chain(),
            ),
        )
        .run();
}

#[derive(Resource)]
pub struct GameState {
    pub score: u32,
    pub n_enemies: u32,
    pub timer: Timer,
    pub last_spawn_time: f32,
    pub game_over: bool,
    pub game_duration: f32, // Total game time in seconds (120.0)
}

/// Preloaded player sprite sheet handles to prevent flickering during state transitions
#[derive(Resource)]
pub struct PlayerSpriteSheets {
    idle: Handle<Image>,
    idle_to_walk: Handle<Image>,
    idle_to_run: Handle<Image>,
    walk: Handle<Image>,
    run: Handle<Image>,
    jump: Handle<Image>,
    falling: Handle<Image>,
    landing: Handle<Image>,
    punch: Handle<Image>,
    punch_combo: Handle<Image>,
    kick: Handle<Image>,
    kick_combo: Handle<Image>,
    punch_kick_combo: Handle<Image>,
    jump_punch: Handle<Image>,
    jump_kick: Handle<Image>,
    defeat: Handle<Image>,
}

impl PlayerSpriteSheets {
    /// Get the sprite handle for a given sprite path
    fn get_handle(&self, sprite_path: &str) -> Handle<Image> {
        match sprite_path {
            "player/idle-sheet.png" => self.idle.clone(),
            "player/idle-to-walk-sheet.png" => self.idle_to_walk.clone(),
            "player/idle-to-run-sheet.png" => self.idle_to_run.clone(),
            "player/walk-sheet.png" => self.walk.clone(),
            "player/run-sheet.png" => self.run.clone(),
            "player/jump-sheet.png" => self.jump.clone(),
            "player/falling-sheet.png" => self.falling.clone(),
            "player/landing-sheet.png" => self.landing.clone(),
            "player/punch-sheet.png" => self.punch.clone(),
            "player/punch-combo-sheet.png" => self.punch_combo.clone(),
            "player/kick-sheet.png" => self.kick.clone(),
            "player/kick-combo-sheet.png" => self.kick_combo.clone(),
            "player/punch-kick-combo-sheet.png" => self.punch_kick_combo.clone(),
            "player/jump-punch-sheet.png" => self.jump_punch.clone(),
            "player/jump-kick-sheet.png" => self.jump_kick.clone(),
            "player/defeat-sheet.png" => self.defeat.clone(),
            _ => panic!("Unknown sprite path: {}", sprite_path),
        }
    }
}

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(
        &AnimationIndices,
        &mut AnimationTimer,
        &mut Sprite,
        Option<&PlayerState>,
    )>,
) {
    for (indices, mut timer, mut sprite, player_state) in &mut query {
        timer.tick(time.delta());

        if timer.just_finished()
            && let Some(atlas) = &mut sprite.texture_atlas
        {
            // Check if player is in Defeat state - freeze on last frame
            let is_defeated =
                player_state.map_or(false, |state| matches!(state, PlayerState::Defeat(_)));

            if is_defeated && atlas.index == indices.last {
                // Freeze on last frame of defeat animation
                atlas.index = indices.last;
            } else {
                // Normal animation loop
                atlas.index = if atlas.index == indices.last {
                    indices.first
                } else {
                    atlas.index + 1
                };
            }
        }
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands.spawn(Camera2d);
    commands.spawn(Sprite::from_image(asset_server.load("graveyard.png")));
    commands.spawn((
        Sprite::from_image(asset_server.load("graveyard_foreground.png")),
        Transform::from_xyz(0., 0., 2.0),
    ));

    // Play background music on loop
    commands.spawn((
        AudioPlayer::<AudioSource>(asset_server.load("music/pixel_showdown.mp3")),
        PlaybackSettings::LOOP,
    ));

    // Preload all player sprite sheets to prevent flickering during transitions
    let sprite_sheets = PlayerSpriteSheets {
        idle: asset_server.load("player/idle-sheet.png"),
        idle_to_walk: asset_server.load("player/idle-to-walk-sheet.png"),
        idle_to_run: asset_server.load("player/idle-to-run-sheet.png"),
        walk: asset_server.load("player/walk-sheet.png"),
        run: asset_server.load("player/run-sheet.png"),
        jump: asset_server.load("player/jump-sheet.png"),
        falling: asset_server.load("player/falling-sheet.png"),
        landing: asset_server.load("player/landing-sheet.png"),
        punch: asset_server.load("player/punch-sheet.png"),
        punch_combo: asset_server.load("player/punch-combo-sheet.png"),
        kick: asset_server.load("player/kick-sheet.png"),
        kick_combo: asset_server.load("player/kick-combo-sheet.png"),
        punch_kick_combo: asset_server.load("player/punch-kick-combo-sheet.png"),
        jump_punch: asset_server.load("player/jump-punch-sheet.png"),
        jump_kick: asset_server.load("player/jump-kick-sheet.png"),
        defeat: asset_server.load("player/defeat-sheet.png"),
    };

    commands.spawn((
        Sprite::from_atlas_image(
            sprite_sheets.idle.clone(),
            TextureAtlas {
                layout: texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
                    UVec2::splat(320),
                    24,
                    1,
                    None,
                    None,
                )),
                index: 1,
            },
        ),
        Transform::from_xyz(-200., -200., 1.),
        Direction::None,
        AnimationIndices { first: 1, last: 23 },
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        PlayerState::transition_to(PlayerStateType::Idle),
        Player,
        Health {
            current: 20,
            max: 20,
        },
        HurtBox {
            size: Vec2::new(100.0, 150.0),
        },
        Hitbox {
            offset: Vec2::ZERO,
            size: Vec2::ZERO,
            active: false,
        },
        HitTracking::default(),
        ComboWindow {
            timer: Timer::from_seconds(0.5, TimerMode::Once),
            last_attack: None,
            queued_combo: None,
        },
        JumpPhysics {
            velocity_y: 0.0,
            ground_y: -100.0,
            jump_force: 1000.0,
            has_used_aerial_attack: false,
        },
    ));

    commands.insert_resource(GameState {
        score: 0,
        n_enemies: 0,
        timer: Timer::from_seconds(120.0, TimerMode::Once),
        last_spawn_time: 0.0,
        game_over: false,
        game_duration: 120.0,
    });

    // Insert preloaded sprite sheets as a resource
    commands.insert_resource(sprite_sheets);

    // Spawn UI elements
    // Score text - top left
    commands.spawn((
        Text::new("Score: 0"),
        TextFont {
            font_size: 30.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            top: Val::Px(20.0),
            ..default()
        },
        ScoreText,
    ));

    // Health text - top center
    commands.spawn((
        Text::new("Health: 20"),
        TextFont {
            font_size: 30.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.3, 0.3)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(45.0),
            top: Val::Px(20.0),
            ..default()
        },
        HealthText,
    ));

    // Time text - top right
    commands.spawn((
        Text::new("Time: 120"),
        TextFont {
            font_size: 30.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(20.0),
            top: Val::Px(20.0),
            ..default()
        },
        TimeText,
    ));
}

fn count_down(time: Res<Time>, mut game_state: ResMut<GameState>) {
    if game_state.game_over {
        return;
    }

    game_state.timer.tick(time.delta());
    if game_state.timer.is_finished() && !game_state.game_over {
        game_state.game_over = true;
        println!("Time's up! Final score: {}", game_state.score);
    }
}

fn spawn_enemy(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut game_state: ResMut<GameState>,
) {
    if game_state.game_over {
        return;
    }
    if game_state.n_enemies >= MAX_ENEMIES {
        return;
    }
    if game_state.timer.elapsed_secs() - game_state.last_spawn_time < 2.0 {
        return;
    }

    // Randomize spawn side
    let spawn_left = rand::random::<bool>();
    let spawn_x = if spawn_left { -1600.0 } else { 1600.0 };
    let direction = if spawn_left {
        Direction::Right
    } else {
        Direction::Left
    };

    commands.spawn((
        Sprite::from_atlas_image(
            asset_server.load("enemies/ghost-sheet.png"),
            TextureAtlas {
                layout: texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
                    UVec2::splat(160),
                    12,
                    1,
                    None,
                    None,
                )),
                index: 1,
            },
        ),
        Transform::from_xyz(spawn_x, 0.0, 2.0).with_scale(Vec3::splat(1.5)),
        direction,
        AnimationIndices { first: 1, last: 11 },
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        EnemyState::Move,
        Enemy,
        Health { current: 6, max: 6 },
        HurtBox {
            size: Vec2::new(80.0, 100.0),
        },
    ));
    game_state.n_enemies += 1;
    game_state.last_spawn_time = game_state.timer.elapsed_secs();
}

fn move_enemies(
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    mut enemy_query: Query<
        (&mut Direction, &mut Transform, &mut Sprite),
        (With<Enemy>, Without<Stunned>, Without<Player>),
    >,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    for (mut dir, mut transform, mut sprite) in enemy_query.iter_mut() {
        // Determine horizontal direction with hysteresis (avoid rapid switching)
        let x_diff = player_transform.translation.x - transform.translation.x;
        if x_diff > 150.0 {
            *dir = Direction::Right;
        } else if x_diff < -150.0 {
            *dir = Direction::Left;
        }
        // Keep current direction if within threshold

        // Move vertically toward player
        let y_diff = player_transform.translation.y - transform.translation.y;
        if y_diff > 10.0 {
            transform.translation.y += 50. * time.delta_secs();
        } else if y_diff < -10.0 {
            transform.translation.y -= 50. * time.delta_secs();
        }

        // Move horizontally based on direction
        match *dir {
            Direction::Right => {
                transform.translation.x += 150. * time.delta_secs();
                sprite.flip_x = false;
            }
            Direction::Left => {
                transform.translation.x -= 150. * time.delta_secs();
                sprite.flip_x = true;
            }
            Direction::None => { /* Do nothing */ }
        }
    }
}

// Combat Systems
fn update_attack_hitboxes(
    mut player_query: Query<
        (
            &PlayerState,
            &Sprite,
            &AnimationIndices,
            &mut Hitbox,
            &Direction,
        ),
        With<Player>,
    >,
) {
    for (state, sprite, indices, mut hitbox, direction) in player_query.iter_mut() {
        if !state.is_attacking() {
            hitbox.active = false;
            continue;
        }

        // Hitbox is active during middle frames of attack animation
        if let Some(atlas) = &sprite.texture_atlas {
            let frame = atlas.index;
            let total_frames = indices.last - indices.first;
            let mid_start = indices.first + (total_frames / 3);
            let mid_end = indices.first + (2 * total_frames / 3);

            hitbox.active = frame >= mid_start && frame <= mid_end;

            // Position hitbox in front of player
            hitbox.offset = match *direction {
                Direction::Right => Vec2::new(80.0, 0.0),
                Direction::Left => Vec2::new(-80.0, 0.0),
                _ => Vec2::ZERO,
            };

            // Size varies by attack type
            hitbox.size = match *state {
                PlayerState::Punch(_) | PlayerState::PunchCombo(_) => Vec2::new(60.0, 40.0),
                PlayerState::Kick(_)
                | PlayerState::KickCombo(_)
                | PlayerState::PunchKickCombo(_) => Vec2::new(80.0, 50.0),
                PlayerState::JumpPunch(_) => Vec2::new(50.0, 50.0),
                PlayerState::JumpKick(_) => Vec2::new(70.0, 60.0),
                _ => Vec2::ZERO,
            };
        }
    }
}

fn aabb_collision(pos1: Vec2, size1: Vec2, pos2: Vec2, size2: Vec2) -> bool {
    let half1 = size1 / 2.0;
    let half2 = size2 / 2.0;
    (pos1.x - half1.x < pos2.x + half2.x)
        && (pos1.x + half1.x > pos2.x - half2.x)
        && (pos1.y - half1.y < pos2.y + half2.y)
        && (pos1.y + half1.y > pos2.y - half2.y)
}

fn detect_combat_collisions(
    mut player_query: Query<
        (Entity, &Transform, &Hitbox, &PlayerState, &mut HitTracking),
        With<Player>,
    >,
    enemy_query: Query<(Entity, &Transform, &HurtBox), With<Enemy>>,
    mut damage_events: MessageWriter<DamageEvent>,
) {
    for (player_entity, player_transform, hitbox, player_state, mut hit_tracking) in
        player_query.iter_mut()
    {
        if !hitbox.active {
            continue;
        }

        let hitbox_center = player_transform.translation.truncate() + hitbox.offset;

        for (enemy_entity, enemy_transform, hurtbox) in enemy_query.iter() {
            // Skip if this enemy was already hit by current attack
            if hit_tracking.hit_enemies.contains(&enemy_entity) {
                continue;
            }

            let enemy_pos = enemy_transform.translation.truncate();

            // AABB collision detection
            let collision = aabb_collision(hitbox_center, hitbox.size, enemy_pos, hurtbox.size);

            if collision {
                // Mark enemy as hit by this attack
                hit_tracking.hit_enemies.insert(enemy_entity);

                damage_events.write(DamageEvent {
                    attacker: player_entity,
                    target: enemy_entity,
                    damage: player_state.get_damage(),
                });
            }
        }
    }
}

fn detect_player_enemy_collisions(
    player_query: Query<(Entity, &Transform), (With<Player>, Without<Invulnerable>)>,
    enemy_query: Query<(Entity, &Transform), (With<Enemy>, Without<Stunned>)>,
    mut damage_events: MessageWriter<DamageEvent>,
) {
    let Ok((player_entity, player_transform)) = player_query.single() else {
        return;
    };

    let player_pos = player_transform.translation.truncate();

    for (enemy_entity, enemy_transform) in enemy_query.iter() {
        let enemy_pos = enemy_transform.translation.truncate();

        // Simple distance check
        let distance = player_pos.distance(enemy_pos);

        if distance < 100.0 {
            damage_events.write(DamageEvent {
                attacker: enemy_entity,
                target: player_entity,
                damage: 1,
            });
        }
    }
}

// Health & Damage Systems
fn handle_damage_events(
    mut commands: Commands,
    mut damage_events: MessageReader<DamageEvent>,
    mut health_query: Query<&mut Health>,
    transform_query: Query<&Transform>,
    mut enemy_defeated_events: MessageWriter<EnemyDefeatedEvent>,
    mut player_defeated_events: MessageWriter<PlayerDefeatedEvent>,
    enemy_query: Query<(), With<Enemy>>,
    player_query: Query<(), With<Player>>,
) {
    for damage_event in damage_events.read() {
        let Ok(mut health) = health_query.get_mut(damage_event.target) else {
            continue;
        };

        health.current -= damage_event.damage;

        // Calculate knockback direction from attacker to target
        let knockback_dir = if let (Ok(attacker_transform), Ok(target_transform)) = (
            transform_query.get(damage_event.attacker),
            transform_query.get(damage_event.target),
        ) {
            let direction =
                (target_transform.translation - attacker_transform.translation).normalize();
            Vec2::new(direction.x, direction.y)
        } else {
            Vec2::ZERO
        };

        if health.current <= 0 {
            // Check if target is enemy or player
            if enemy_query.get(damage_event.target).is_ok() {
                enemy_defeated_events.write(EnemyDefeatedEvent {
                    enemy: damage_event.target,
                });
            } else if player_query.get(damage_event.target).is_ok() {
                player_defeated_events.write(PlayerDefeatedEvent);
            }
        } else {
            // Entity is still alive - add hit effects
            let is_enemy = enemy_query.get(damage_event.target).is_ok();
            let is_player = player_query.get(damage_event.target).is_ok();

            if is_enemy {
                // Enemy hit but not dead - add stun, knockback, and hit flash
                commands.entity(damage_event.target).insert((
                    Stunned {
                        timer: Timer::from_seconds(0.5, TimerMode::Once),
                    },
                    Knockback {
                        velocity: knockback_dir * 300.0, // Medium knockback for enemies
                    },
                    HitFlash {
                        timer: Timer::from_seconds(0.3, TimerMode::Once),
                        flash_duration: 0.3,
                    },
                ));
            } else if is_player {
                // Player hit - add invulnerability, knockback, and hit flash
                commands.entity(damage_event.target).insert((
                    Invulnerable {
                        timer: Timer::from_seconds(1.0, TimerMode::Once),
                    },
                    Knockback {
                        velocity: knockback_dir * 500.0, // Stronger knockback for player
                    },
                    HitFlash {
                        timer: Timer::from_seconds(0.3, TimerMode::Once),
                        flash_duration: 0.3,
                    },
                ));
            }
        }
    }
}

fn update_stun_timers(
    mut commands: Commands,
    time: Res<Time>,
    mut stunned_query: Query<(Entity, &mut Stunned)>,
) {
    for (entity, mut stunned) in stunned_query.iter_mut() {
        stunned.timer.tick(time.delta());

        if stunned.timer.is_finished() {
            commands.entity(entity).remove::<Stunned>();
        }
    }
}

fn handle_enemy_defeat(
    mut commands: Commands,
    mut events: MessageReader<EnemyDefeatedEvent>,
    mut game_state: ResMut<GameState>,
) {
    for event in events.read() {
        commands.entity(event.enemy).despawn();
        game_state.n_enemies -= 1;
        game_state.score += 10;
    }
}

fn handle_player_defeat(
    mut events: MessageReader<PlayerDefeatedEvent>,
    mut game_state: ResMut<GameState>,
    mut player_query: Query<&mut PlayerState, With<Player>>,
) {
    for _event in events.read() {
        // Transition player to Defeat state
        if let Ok(mut state) = player_query.single_mut() {
            *state = PlayerState::transition_to(PlayerStateType::Defeat);
        }

        // Set game over flag
        game_state.game_over = true;
        println!("GAME OVER! Final Score: {}", game_state.score);
    }
}

/// Apply knockback velocity to entities and decay it over time
fn apply_knockback(
    mut commands: Commands,
    time: Res<Time>,
    mut knockback_query: Query<(Entity, &mut Transform, &mut Knockback, Option<&JumpPhysics>)>,
) {
    for (entity, mut transform, mut knockback, jump_physics) in knockback_query.iter_mut() {
        // For grounded players, only apply horizontal knockback
        let is_grounded = jump_physics.map_or(false, |jp| {
            (transform.translation.y - jp.ground_y).abs() < 1.0
        });

        if is_grounded {
            // Only apply horizontal knockback for grounded entities
            transform.translation.x += knockback.velocity.x * time.delta_secs();
        } else {
            // Apply full knockback (both X and Y) for airborne/enemy entities
            transform.translation.x += knockback.velocity.x * time.delta_secs();
            transform.translation.y += knockback.velocity.y * time.delta_secs();
        }

        // Decay knockback velocity (friction)
        knockback.velocity *= 0.9;

        // Remove knockback component when velocity is negligible
        if knockback.velocity.length() < 10.0 {
            commands.entity(entity).remove::<Knockback>();
        }
    }
}

/// Update hit flash effect - creates red glow by modulating sprite color
fn update_hit_flash(
    mut commands: Commands,
    time: Res<Time>,
    mut flash_query: Query<(Entity, &mut Sprite, &mut HitFlash)>,
) {
    for (entity, mut sprite, mut hit_flash) in flash_query.iter_mut() {
        hit_flash.timer.tick(time.delta());

        // Calculate flash intensity (starts at 1.0, fades to 0.0)
        let progress = hit_flash.timer.elapsed_secs() / hit_flash.flash_duration;
        let intensity = (1.0 - progress).max(0.0);

        // Apply red tint (white + red = lighter red glow)
        sprite.color = Color::srgb(1.0, 1.0 - intensity * 0.7, 1.0 - intensity * 0.7);

        if hit_flash.timer.is_finished() {
            // Reset color to white and remove component
            sprite.color = Color::WHITE;
            commands.entity(entity).remove::<HitFlash>();
        }
    }
}

/// Update invulnerability timer and remove when expired
fn update_invulnerability(
    mut commands: Commands,
    time: Res<Time>,
    mut invuln_query: Query<(Entity, &mut Invulnerable)>,
) {
    for (entity, mut invulnerable) in invuln_query.iter_mut() {
        invulnerable.timer.tick(time.delta());

        if invulnerable.timer.is_finished() {
            commands.entity(entity).remove::<Invulnerable>();
        }
    }
}

/// Update UI elements with current game state
fn update_ui(
    game_state: Res<GameState>,
    player_query: Query<&Health, With<Player>>,
    mut score_text: Query<&mut Text, (With<ScoreText>, Without<HealthText>, Without<TimeText>)>,
    mut health_text: Query<&mut Text, (With<HealthText>, Without<ScoreText>, Without<TimeText>)>,
    mut time_text: Query<&mut Text, (With<TimeText>, Without<ScoreText>, Without<HealthText>)>,
) {
    // Update score
    if let Ok(mut text) = score_text.single_mut() {
        **text = format!("Score: {}", game_state.score);
    }

    // Update health
    if let Ok(health) = player_query.single() {
        if let Ok(mut text) = health_text.single_mut() {
            **text = format!("Health: {}", health.current);
        }
    }

    // Update time remaining
    if let Ok(mut text) = time_text.single_mut() {
        let time_remaining = (game_state.game_duration - game_state.timer.elapsed_secs()).max(0.0);
        **text = format!("Time: {}", time_remaining.ceil() as u32);
    }
}

/// Handle game over state - despawn enemies and show game over screen
fn handle_game_over(
    mut commands: Commands,
    game_state: Res<GameState>,
    enemy_query: Query<Entity, With<Enemy>>,
    game_over_query: Query<&GameOverScreen>,
) {
    if !game_state.game_over {
        return;
    }

    // Check if game over screen already exists
    if game_over_query.is_empty() {
        // Despawn all enemies
        for enemy_entity in enemy_query.iter() {
            commands.entity(enemy_entity).despawn();
        }

        // Spawn game over screen
        commands.spawn((
            Text::new("GAME OVER"),
            TextFont {
                font_size: 80.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 0.0, 0.0)),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(35.0),
                top: Val::Percent(40.0),
                ..default()
            },
            GameOverScreen,
        ));

        // Spawn final score text
        commands.spawn((
            Text::new(format!("Final Score: {}", game_state.score)),
            TextFont {
                font_size: 40.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(38.0),
                top: Val::Percent(50.0),
                ..default()
            },
            GameOverScreen,
        ));

        // Spawn restart instruction text
        commands.spawn((
            Text::new("Press R to Restart"),
            TextFont {
                font_size: 30.0,
                ..default()
            },
            TextColor(Color::srgb(0.7, 0.7, 0.7)),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(40.0),
                top: Val::Percent(60.0),
                ..default()
            },
            GameOverScreen,
        ));
    }
}

/// Handle restart input - reset game state when R is pressed during game over
fn handle_restart(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut game_state: ResMut<GameState>,
    mut player_query: Query<
        (
            Entity,
            &mut Health,
            &mut PlayerState,
            &mut Transform,
            &mut JumpPhysics,
            &mut ComboWindow,
        ),
        With<Player>,
    >,
    game_over_screen_query: Query<Entity, With<GameOverScreen>>,
    enemy_query: Query<Entity, With<Enemy>>,
) {
    // Only process restart when game is over
    if !game_state.game_over {
        return;
    }

    // Check for R key press
    if keyboard.just_pressed(KeyCode::KeyR) {
        // Reset game state
        game_state.score = 0;
        game_state.n_enemies = 0;
        game_state.timer = Timer::from_seconds(120.0, TimerMode::Once);
        game_state.last_spawn_time = 0.0;
        game_state.game_over = false;

        // Reset player
        if let Ok((
            player_entity,
            mut health,
            mut state,
            mut transform,
            mut jump_physics,
            mut combo_window,
        )) = player_query.single_mut()
        {
            // Reset health
            health.current = health.max;

            // Reset state to Idle
            *state = PlayerState::transition_to(PlayerStateType::Idle);

            // Reset position
            transform.translation = Vec3::new(-200.0, -200.0, 1.0);

            // Reset jump physics
            jump_physics.velocity_y = 0.0;
            jump_physics.has_used_aerial_attack = false;

            // Reset combo window
            combo_window.timer = Timer::from_seconds(0.5, TimerMode::Once);
            combo_window.last_attack = None;
            combo_window.queued_combo = None;

            // Remove any active combat effects
            commands
                .entity(player_entity)
                .remove::<Invulnerable>()
                .remove::<Knockback>()
                .remove::<HitFlash>();
        }

        // Despawn game over UI
        for entity in game_over_screen_query.iter() {
            commands.entity(entity).despawn();
        }

        // Despawn all enemies
        for enemy_entity in enemy_query.iter() {
            commands.entity(enemy_entity).despawn();
        }

        println!("Game restarted!");
    }
}
