# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Martial Magicka is a 2D side-scrolling martial arts game built with Bevy 0.17.3 (Rust game engine). The game features a player character with a complete combat state machine (punch, kick, combos) fighting against AI-controlled ghost enemies that spawn from both sides of the screen.

## Build and Development Commands

```bash
# Run the game in development mode (with hot reloading via dynamic linking)
cargo run

# Build release version
cargo build --release

# Run release build
cargo run --release

# Check for compilation errors without building
cargo check

# Format code
cargo fmt

# Run linter
cargo clippy
```

## Architecture

### ECS Pattern (Entity Component System)

This project uses Bevy's ECS architecture exclusively. All game logic is organized into:

- **Components**: Data structures attached to entities (see src/main.rs:68-196)
  - Basic: `Direction`, `AnimationIndices`, `AnimationTimer`, `PlayerState`, `EnemyState`
  - Combat: `Health`, `Hitbox`, `HurtBox`, `Stunned`, `ComboWindow`
  - Markers: `Player`, `Enemy` (for query filtering)
- **Systems**: Pure functions that query components and operate on entities
- **Resources**: Global state (`GameState` tracks score, enemy count, spawn timing)
- **Messages**: Event-driven communication (`DamageEvent`, `EnemyDefeatedEvent`, `PlayerDefeatedEvent`)

**Important**: Bevy 0.17.3 renamed `Event` to `Message`. Use `#[derive(Message)]` and `MessageReader`/`MessageWriter` instead of `EventReader`/`EventWriter`.

### System Execution Order

Systems are strictly ordered using `.chain()` to prevent race conditions (src/main.rs:29-55):

1. **Input & State Management**: `handle_player_input` → `manage_state_transitions` → `update_player_sprite_from_state`
2. **Movement**: `handle_player_movement` (player), `move_enemies` (AI)
3. **Combat & Collision**: `update_attack_hitboxes` → `detect_combat_collisions` → `detect_player_enemy_collisions`
4. **Damage Resolution**: `handle_damage_events` → `update_stun_timers` → `handle_enemy_defeat` → `handle_player_defeat`
5. **Animation & Game Management**: `animate_sprite`, `count_down`, `spawn_enemy`

This ordering ensures:
- State changes occur before sprite updates
- Hitboxes are positioned before collision detection
- Damage is applied before checking for defeats
- Movement happens with current frame's state

### Player State Machine

The player has 15 distinct states (src/main.rs:88-103):

**Movement States**:
- `Idle`, `IdleToWalk`, `IdleToRun`, `Walk`, `Run`
- `Jump`, `Fall`, `Land`

**Combat States**:
- `Punch`, `PunchCombo` (double-tap up arrow)
- `Kick`, `KickCombo` (double-tap down arrow)
- `PunchKickCombo` (punch combo → down arrow)
- `JumpPunch`, `JumpKick` (aerial attacks)

**State Transition Logic**:
- Animation-driven: Attack states return to `Idle` when animation completes
- Jump sequence: `Jump` → `Fall` → `Land` → `Idle`
- Combo window: 0.5 seconds to chain attacks
- Input locking: Movement disabled during attack animations (`locks_input()`)

### Combat System

**Hitbox Architecture**:
- Player has `Hitbox` component (position, size, active flag)
- Enemies have `HurtBox` component (collision area)
- Hitboxes only active during middle third of attack animation frames
- AABB (Axis-Aligned Bounding Box) collision detection

**Damage System**:
- Punches: 2 damage
- Kicks: 3 damage
- Ghosts: 6 HP (3 punches or 2 kicks to defeat)
- Player: 20 HP
- Enemies pause (stunned) for 0.5s when hit

**Event-Driven Flow**:
1. Collision detection systems write `DamageEvent` messages
2. `handle_damage_events` reads messages and applies damage
3. If HP ≤ 0, writes `EnemyDefeatedEvent` or `PlayerDefeatedEvent`
4. Defeat handlers despawn entities and update score

### Enemy AI

Enemies spawn every 2 seconds (max 6 active):
- Random spawn side: left (-2000, 0) or right (2000, 0) using `rand::random::<bool>()`
- Movement: 150 units/sec horizontal, 50 units/sec vertical
- Direction hysteresis: 150-pixel threshold prevents rapid direction switching
- Tracking: Moves toward player position on both X and Y axes
- Stun mechanic: `Without<Stunned>` filter in `move_enemies` query pauses movement when hit

### Animation System

**Sprite Sheet Specifications**:
- Player sprites: **320x320 pixel tiles** (resized from 640x640 to avoid GPU limits)
- Variable frame counts per animation (see `get_animation_indices()`)
- Enemy sprites: 160x160 tiles, 12 frames
- Animation speed: 0.1s per frame (10 FPS)

**GPU Texture Limit Constraint**:
- Maximum texture width: 16,384 pixels (GPU hardware limit)
- Jump animation has 27 frames: 27 × 320 = 8,640 pixels ✓
- Original 640×640 tiles exceeded limit: 27 × 640 = 17,280 pixels ✗

**Dynamic Sprite Sheet Swapping**:
- `update_player_sprite_from_state` uses `Changed<PlayerState>` filter
- Creates new `TextureAtlasLayout` with correct column count for each animation
- Resets atlas index to first frame BEFORE changing layout (prevents out-of-bounds access)
- Loads new sprite image AFTER atlas is configured (prevents blinking)

### Query Conflict Resolution

Player and enemy queries must be disjoint to avoid ECS conflicts:
- `handle_player_movement`: `(With<Player>, Without<Enemy>)`
- `move_enemies`: `(With<Enemy>, Without<Stunned>, Without<Player>)`

Without these filters, both systems would try to mutably access `Transform` and `Sprite` simultaneously, causing panic B0001.

## Asset Structure

```
assets/
├── desert.png              # Background sprite
├── player/
│   ├── idle-sheet.png      # 24 frames (7,680px wide)
│   ├── walk-sheet.png      # 12 frames
│   ├── run-sheet.png       # 8 frames
│   ├── jump-sheet.png      # 27 frames (8,640px wide)
│   ├── punch-sheet.png     # 13 frames
│   ├── kick-sheet.png      # 21 frames
│   └── ... (17 total)      # All 320x320 tiles
└── enemies/
    └── ghost-sheet.png     # 12 frames, 160x160 tiles
```

**Frame Count Calculation**: width / tile_size (e.g., 7680 / 320 = 24 frames)

## Controls

- **A**: Run left
- **D**: Run right
- **Shift + A/D**: Walk (slower movement)
- **Space**: Jump
- **Up Arrow**: Punch (double-tap for combo)
- **Down Arrow**: Kick (double-tap for combo, or after punch combo for mixed combo)

## Development Notes

### Bevy 0.17.3 Specifics

- Uses `dynamic_linking` feature for faster compile times in dev mode
- Dev profile: opt-level 1 for project code, opt-level 3 for dependencies
- Edition 2024 Rust features enabled
- Messages API (not Events API): Use `MessageReader`/`MessageWriter`, iterate with `for event in reader.read()`

### Common Pitfalls

1. **Query conflicts**: Always use `Without<T>` to make queries disjoint when multiple systems access same components
2. **Sprite blinking**: Set atlas index to first frame BEFORE creating new layout
3. **State machine loops**: Ensure all state paths eventually return to `Idle` or another stable state
4. **GPU texture limits**: Keep sprite sheets under 16,384 pixels wide (51 frames max at 320px tiles)
5. **Direction hysteresis**: Use threshold zones (e.g., 150px) to prevent rapid switching at boundaries

### Debugging Animation Issues

If sprites blink or disappear:
1. Check frame counts match sprite sheet width (divide by 320)
2. Verify `get_animation_indices()` returns correct (first, last) for each state
3. Ensure atlas index is reset before changing layout in `update_player_sprite_from_state`
4. Confirm texture atlas column count matches actual frames: `(last + 1) as u32`
