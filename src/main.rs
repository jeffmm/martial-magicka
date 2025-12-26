//! Renders a 2D scene containing a single, moving sprite.

use bevy::prelude::*;
const MAX_ENEMIES: u32 = 6;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                animate_sprite,
                keyboard_input_system,
                count_down,
                spawn_enemy,
                move_enemies,
            ),
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

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

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
                    UVec2::splat(640),
                    16,
                    1,
                    None,
                    None,
                )),
                index: 1,
            },
        ),
        Transform::from_xyz(-200., -200., 1.).with_scale(Vec3::splat(0.5)),
        Direction::Right,
        AnimationIndices { first: 1, last: 15 },
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        PlayerState::Idle,
        PlayerState::Idle,
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
        Transform::from_xyz(-2000.0, 0.0, 2.0),
        Direction::Right,
        AnimationIndices { first: 1, last: 11 },
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        EnemyState::Move,
    ));
    game_state.n_enemies += 1;
    game_state.last_spawn_time = game_state.timer.elapsed_secs();
}

fn move_enemies(
    time: Res<Time>,
    player_sprite: Query<(&Transform, &PlayerState)>,
    mut enemy_sprite: Query<(&mut Direction, &mut Transform, &mut Sprite, &EnemyState)>,
) {
    // Get next player transform from player_sprite query
    let player = player_sprite.single();
    if !player.is_ok() {
        return;
    }
    let (player_transform, _player_state) = player.unwrap();

    for (mut dir, mut transform, mut sprite, _state) in &mut enemy_sprite {
        if player_transform.translation.x - transform.translation.x > 100.0 {
            *dir = Direction::Right;
        } else {
            *dir = Direction::Left;
        }
        if player_transform.translation.y - transform.translation.y > 10.0 {
            transform.translation.y += 50. * time.delta_secs();
        }
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
    mut player_sprite: Query<(
        &mut AnimationIndices,
        &mut AnimationTimer,
        &mut Sprite,
        &PlayerState,
    )>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut game_state: ResMut<GameState>,
) {
    // Get next player transform from player_sprite query
    let player = player_sprite.single();
    if !player.is_ok() {
        return;
    }
    let (mut indices, mut timer, mut sprite, state) = player.unwrap();
    if game_state.prev_player_state == *state {
        return;
    }

    match state {
        PlayerState::Idle => {
            // Update atlas image for sprite
        }
        PlayerState::Walk => {
            // Update atlas image for sprite for walk animation
        }
        _ => { /* Handle other states as needed */ }
    }
    game_state.prev_player_state = *state;
}

/// This system responds to certain key presses
fn keyboard_input_system(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_sprite: Query<(
        &mut Direction,
        &mut Transform,
        &mut Sprite,
        &mut PlayerState,
    )>,
) {
    for (mut dir, mut transform, mut sprite, mut state) in &mut player_sprite {
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
        if keyboard_input.pressed(KeyCode::KeyA) {
            *dir = Direction::Left;
        } else if keyboard_input.pressed(KeyCode::KeyD) {
            *dir = Direction::Right;
        } else {
            *dir = Direction::None;
        }
    }
}
