//! Renders a 2D scene containing a single, moving sprite.

use bevy::prelude::*;
const MAX_ENEMIES: u32 = 6;

// Messages (Events)
#[derive(Message)]
struct DamageEvent {
    attacker: Entity,
    target: Entity,
    damage: i32,
}

#[derive(Message)]
struct EnemyDefeatedEvent {
    enemy: Entity,
}

#[derive(Message)]
struct PlayerDefeatedEvent;

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
                // Phase 1: Input & State Management
                handle_player_input,
                manage_state_transitions,
                update_player_sprite_from_state,
                // Phase 2: Movement
                handle_player_movement,
                // Phase 3: Combat & Collision
                update_attack_hitboxes,
                detect_combat_collisions,
                detect_player_enemy_collisions,
                // Phase 4: Damage Resolution
                handle_damage_events,
                update_stun_timers,
                handle_enemy_defeat,
                handle_player_defeat,
                // Phase 5: Enemy AI
                move_enemies,
                // Phase 6: Animation & Game Management
                animate_sprite,
                count_down,
                spawn_enemy,
            )
                .chain(),
        )
        .run();
}

#[derive(Resource)]
struct GameState {
    score: u32,
    n_enemies: u32,
    timer: Timer,
    last_spawn_time: f32,
    prev_player_state: PlayerState,
}

#[derive(Component)]
enum Direction {
    None,
    Left,
    Right,
}

#[derive(Component)]
struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Component)]
enum EnemyState {
    Idle,
    Move,
}

#[derive(Component, PartialEq, Eq, Clone, Copy)]
enum PlayerState {
    Idle,
    IdleToWalk,
    IdleToRun,
    Walk,
    Run,
    Jump,
    Fall,
    Land,
    Punch,
    PunchCombo,
    Kick,
    KickCombo,
    PunchKickCombo,
    JumpPunch,
    JumpKick,
}

impl PlayerState {
    fn is_attacking(&self) -> bool {
        matches!(
            self,
            PlayerState::Punch
                | PlayerState::PunchCombo
                | PlayerState::Kick
                | PlayerState::KickCombo
                | PlayerState::PunchKickCombo
                | PlayerState::JumpPunch
                | PlayerState::JumpKick
        )
    }

    fn locks_input(&self) -> bool {
        self.is_attacking()
    }

    fn get_sprite_path(&self) -> &'static str {
        match self {
            PlayerState::Idle => "player/idle-sheet.png",
            PlayerState::IdleToWalk => "player/idle-to-walk-sheet.png",
            PlayerState::IdleToRun => "player/idle-to-run-sheet.png",
            PlayerState::Walk => "player/walk-sheet.png",
            PlayerState::Run => "player/run-sheet.png",
            PlayerState::Jump => "player/jump-sheet.png",
            PlayerState::Fall => "player/falling-sheet.png",
            PlayerState::Land => "player/landing-sheet.png",
            PlayerState::Punch => "player/punch-sheet.png",
            PlayerState::PunchCombo => "player/punch-combo-sheet.png",
            PlayerState::Kick => "player/kick-sheet.png",
            PlayerState::KickCombo => "player/kick-combo-sheet.png",
            PlayerState::PunchKickCombo => "player/punch-kick-combo-sheet.png",
            PlayerState::JumpPunch => "player/jump-punch-sheet.png",
            PlayerState::JumpKick => "player/jump-kick-sheet.png",
        }
    }

    fn get_animation_indices(&self) -> (usize, usize) {
        // Frame counts based on actual sprite sheet dimensions (width / 320)
        // First frame is 1 (skipping frame 0 which is often blank)
        match self {
            PlayerState::Idle => (1, 23),            // 24 frames total
            PlayerState::IdleToWalk => (1, 6),       // 7 frames total
            PlayerState::IdleToRun => (1, 7),        // 8 frames total
            PlayerState::Walk => (1, 11),            // 12 frames total
            PlayerState::Run => (1, 7),              // 8 frames total
            PlayerState::Jump => (1, 26),            // 27 frames total
            PlayerState::Fall => (1, 19),            // 20 frames total
            PlayerState::Land => (1, 20),            // 21 frames total
            PlayerState::Punch => (1, 12),           // 13 frames total
            PlayerState::PunchCombo => (1, 7),       // 8 frames total
            PlayerState::Kick => (1, 20),            // 21 frames total
            PlayerState::KickCombo => (1, 19),       // 20 frames total
            PlayerState::PunchKickCombo => (1, 16),  // 17 frames total
            PlayerState::JumpPunch => (1, 17),       // 18 frames total
            PlayerState::JumpKick => (1, 19),        // 20 frames total
        }
    }

    fn get_damage(&self) -> i32 {
        match self {
            PlayerState::Punch | PlayerState::PunchCombo | PlayerState::JumpPunch => 2,
            PlayerState::Kick
            | PlayerState::KickCombo
            | PlayerState::JumpKick
            | PlayerState::PunchKickCombo => 3,
            _ => 0,
        }
    }
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

// Combat Components
#[derive(Component)]
struct Health {
    current: i32,
    max: i32,
}

#[derive(Component)]
struct Hitbox {
    offset: Vec2,
    size: Vec2,
    active: bool,
}

#[derive(Component)]
struct HurtBox {
    size: Vec2,
}

#[derive(Component)]
struct Stunned {
    timer: Timer,
}

#[derive(Component)]
struct ComboWindow {
    timer: Timer,
    last_attack: Option<PlayerState>,
}

// Marker Components
#[derive(Component)]
struct Player;

#[derive(Component)]
struct Enemy;

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(&AnimationIndices, &mut AnimationTimer, &mut Sprite)>,
) {
    for (indices, mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());

        if timer.just_finished()
            && let Some(atlas) = &mut sprite.texture_atlas
        {
            atlas.index = if atlas.index == indices.last {
                indices.first
            } else {
                atlas.index + 1
            };
        }
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands.spawn(Camera2d);
    commands.spawn(Sprite::from_image(asset_server.load("desert.png")));
    commands.spawn((
        Sprite::from_atlas_image(
            asset_server.load("player/idle-sheet.png"),
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
        PlayerState::Idle,
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
        ComboWindow {
            timer: Timer::from_seconds(0.5, TimerMode::Once),
            last_attack: None,
        },
    ));
    commands.insert_resource(GameState {
        score: 0,
        n_enemies: 0,
        timer: Timer::from_seconds(120.0, TimerMode::Once),
        last_spawn_time: 0.0,
        prev_player_state: PlayerState::Idle,
    });
}

fn count_down(time: Res<Time>, mut game_state: ResMut<GameState>) {
    game_state.timer.tick(time.delta());
    if game_state.timer.is_finished() {
        println!("Time's up! Final score: {}", game_state.score);
    }
}

fn spawn_enemy(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut game_state: ResMut<GameState>,
) {
    if game_state.n_enemies >= MAX_ENEMIES {
        return;
    }
    if game_state.timer.elapsed_secs() - game_state.last_spawn_time < 2.0 {
        return;
    }

    // Randomize spawn side
    let spawn_left = rand::random::<bool>();
    let spawn_x = if spawn_left { -2000.0 } else { 2000.0 };
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
        Transform::from_xyz(spawn_x, 0.0, 2.0),
        direction,
        AnimationIndices { first: 1, last: 11 },
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        EnemyState::Move,
        Enemy,
        Health {
            current: 6,
            max: 6,
        },
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
    mut enemy_query: Query<(&mut Direction, &mut Transform, &mut Sprite), (With<Enemy>, Without<Stunned>, Without<Player>)>,
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

fn update_player_sprite_from_state(
    mut player_query: Query<
        (
            &PlayerState,
            &mut Sprite,
            &mut AnimationIndices,
            &mut AnimationTimer,
        ),
        (With<Player>, Changed<PlayerState>),
    >,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    for (state, mut sprite, mut indices, mut timer) in player_query.iter_mut() {
        // Get new animation parameters
        let sprite_path = state.get_sprite_path();
        let (first, last) = state.get_animation_indices();
        let num_columns = (last + 1) as u32; // +1 because we skip frame 0

        // Update texture atlas FIRST to prevent frame index mismatches
        if let Some(ref mut atlas) = sprite.texture_atlas {
            // Reset to first frame BEFORE changing layout to prevent out-of-bounds access
            atlas.index = first;

            // Create new texture atlas layout with correct number of columns
            atlas.layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
                UVec2::splat(320),
                num_columns,
                1,
                None,
                None,
            ));
        }

        // Update sprite image handle AFTER atlas is configured
        sprite.image = asset_server.load(sprite_path);

        // Update animation indices
        indices.first = first;
        indices.last = last;

        // Reset timer to start animation fresh
        timer.reset();
    }
}

// Input System
fn handle_player_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&mut PlayerState, &mut Direction, &mut ComboWindow), With<Player>>,
) {
    let Ok((mut state, mut direction, mut combo_window)) = player_query.single_mut() else {
        return;
    };

    // Can't change state if locked by animation
    if state.locks_input() {
        return;
    }

    let shift_held =
        keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

    // Combat inputs (highest priority)
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        handle_punch_input(&mut state, &mut combo_window);
        return;
    }

    if keyboard.just_pressed(KeyCode::ArrowDown) {
        handle_kick_input(&mut state, &mut combo_window);
        return;
    }

    // Jump input
    if keyboard.just_pressed(KeyCode::Space) {
        if matches!(
            *state,
            PlayerState::Idle | PlayerState::Run | PlayerState::Walk
        ) {
            *state = PlayerState::Jump;
        }
        return;
    }

    // Movement inputs (lower priority)
    let left = keyboard.pressed(KeyCode::KeyA);
    let right = keyboard.pressed(KeyCode::KeyD);

    if left || right {
        let new_direction = if left {
            Direction::Left
        } else {
            Direction::Right
        };

        match *state {
            PlayerState::Idle => {
                *direction = new_direction;
                *state = if shift_held {
                    PlayerState::IdleToWalk
                } else {
                    PlayerState::IdleToRun
                };
            }
            PlayerState::Walk | PlayerState::Run => {
                *direction = new_direction;
            }
            _ => {}
        }
    } else {
        // No movement keys pressed
        match *state {
            PlayerState::Walk | PlayerState::Run => {
                *state = PlayerState::Idle;
                *direction = Direction::None;
            }
            _ => {}
        }
    }
}

fn handle_punch_input(state: &mut PlayerState, combo_window: &mut ComboWindow) {
    match *state {
        PlayerState::Idle
        | PlayerState::Walk
        | PlayerState::Run
        | PlayerState::IdleToWalk
        | PlayerState::IdleToRun => {
            *state = PlayerState::Punch;
            combo_window.last_attack = Some(PlayerState::Punch);
            combo_window.timer.reset();
        }
        PlayerState::Jump | PlayerState::Fall => {
            *state = PlayerState::JumpPunch;
        }
        PlayerState::Punch => {
            if !combo_window.timer.is_finished() {
                *state = PlayerState::PunchCombo;
                combo_window.last_attack = Some(PlayerState::PunchCombo);
            }
        }
        _ => {}
    }
}

fn handle_kick_input(state: &mut PlayerState, combo_window: &mut ComboWindow) {
    match *state {
        PlayerState::Idle
        | PlayerState::Walk
        | PlayerState::Run
        | PlayerState::IdleToWalk
        | PlayerState::IdleToRun => {
            *state = PlayerState::Kick;
            combo_window.last_attack = Some(PlayerState::Kick);
            combo_window.timer.reset();
        }
        PlayerState::Jump | PlayerState::Fall => {
            *state = PlayerState::JumpKick;
        }
        PlayerState::Kick => {
            if !combo_window.timer.is_finished() {
                *state = PlayerState::KickCombo;
                combo_window.last_attack = Some(PlayerState::KickCombo);
            }
        }
        PlayerState::PunchCombo => {
            if !combo_window.timer.is_finished() {
                *state = PlayerState::PunchKickCombo;
            }
        }
        _ => {}
    }
}

// State Management System
fn manage_state_transitions(
    mut player_query: Query<
        (
            &mut PlayerState,
            &AnimationTimer,
            &AnimationIndices,
            &Sprite,
            &mut ComboWindow,
        ),
        With<Player>,
    >,
    time: Res<Time>,
) {
    for (mut state, timer, indices, sprite, mut combo_window) in player_query.iter_mut() {
        // Tick combo window
        combo_window.timer.tick(time.delta());

        // Check if animation finished (looped back to first frame)
        if let Some(atlas) = &sprite.texture_atlas {
            let animation_finished = atlas.index == indices.last && timer.just_finished();

            if animation_finished {
                match *state {
                    // Attack states return to Idle when animation completes
                    PlayerState::Punch
                    | PlayerState::PunchCombo
                    | PlayerState::Kick
                    | PlayerState::KickCombo
                    | PlayerState::PunchKickCombo
                    | PlayerState::JumpPunch
                    | PlayerState::JumpKick => {
                        *state = PlayerState::Idle;
                        combo_window.last_attack = None;
                    }

                    // Transition states auto-advance
                    PlayerState::IdleToWalk => {
                        *state = PlayerState::Walk;
                    }
                    PlayerState::IdleToRun => {
                        *state = PlayerState::Run;
                    }

                    // Jump → Fall → Land → Idle sequence
                    PlayerState::Jump => {
                        *state = PlayerState::Fall;
                    }
                    PlayerState::Fall => {
                        *state = PlayerState::Land;
                    }
                    PlayerState::Land => {
                        *state = PlayerState::Idle;
                    }

                    _ => {}
                }
            }
        }
    }
}

// Movement System
fn handle_player_movement(
    time: Res<Time>,
    mut player_query: Query<(&PlayerState, &Direction, &mut Transform, &mut Sprite), (With<Player>, Without<Enemy>)>,
) {
    for (state, direction, mut transform, mut sprite) in player_query.iter_mut() {
        // Can't move during attack animations
        if state.locks_input() {
            continue;
        }

        let speed = match *state {
            PlayerState::Walk | PlayerState::IdleToWalk => 75.0,
            PlayerState::Run | PlayerState::IdleToRun => 150.0,
            _ => 0.0,
        };

        match *direction {
            Direction::Left => {
                transform.translation.x -= speed * time.delta_secs();
                sprite.flip_x = true;
            }
            Direction::Right => {
                transform.translation.x += speed * time.delta_secs();
                sprite.flip_x = false;
            }
            Direction::None => {}
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
                PlayerState::Punch | PlayerState::PunchCombo => Vec2::new(60.0, 40.0),
                PlayerState::Kick | PlayerState::KickCombo | PlayerState::PunchKickCombo => {
                    Vec2::new(80.0, 50.0)
                }
                PlayerState::JumpPunch => Vec2::new(50.0, 50.0),
                PlayerState::JumpKick => Vec2::new(70.0, 60.0),
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
    player_query: Query<(Entity, &Transform, &Hitbox, &PlayerState), With<Player>>,
    enemy_query: Query<(Entity, &Transform, &HurtBox), (With<Enemy>, Without<Stunned>)>,
    mut damage_events: MessageWriter<DamageEvent>,
) {
    for (player_entity, player_transform, hitbox, player_state) in player_query.iter() {
        if !hitbox.active {
            continue;
        }

        let hitbox_center = player_transform.translation.truncate() + hitbox.offset;

        for (enemy_entity, enemy_transform, hurtbox) in enemy_query.iter() {
            let enemy_pos = enemy_transform.translation.truncate();

            // AABB collision detection
            let collision = aabb_collision(hitbox_center, hitbox.size, enemy_pos, hurtbox.size);

            if collision {
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
    player_query: Query<(Entity, &Transform), With<Player>>,
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

        if health.current <= 0 {
            // Check if target is enemy or player
            if enemy_query.get(damage_event.target).is_ok() {
                enemy_defeated_events.write(EnemyDefeatedEvent {
                    enemy: damage_event.target,
                });
            } else if player_query.get(damage_event.target).is_ok() {
                player_defeated_events.write(PlayerDefeatedEvent);
            }
        } else if enemy_query.get(damage_event.target).is_ok() {
            // Enemy hit but not dead - add stun
            commands.entity(damage_event.target).insert(Stunned {
                timer: Timer::from_seconds(0.5, TimerMode::Once),
            });
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
    game_state: Res<GameState>,
) {
    for _event in events.read() {
        println!("GAME OVER! Final Score: {}", game_state.score);
    }
}
