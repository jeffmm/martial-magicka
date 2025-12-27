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

### Code Organization

The codebase is organized into modular packages for maintainability:

```
src/
├── main.rs                  # App setup, game loop, core systems
├── player/                  # Player-specific code
│   ├── config.rs           # Animation/physics configs, state types
│   ├── state.rs            # PlayerState enum + StateLogic trait
│   ├── components.rs       # Player, JumpPhysics, ComboWindow
│   ├── systems.rs          # 5 player systems (input, update, sprite, physics)
│   └── states/             # 15 state implementations (one per file)
│       ├── idle.rs
│       ├── movement.rs     # Walk, Run, IdleToWalk, IdleToRun
│       ├── jump.rs         # Jump, Fall, Land
│       ├── punch.rs        # Punch, PunchCombo
│       ├── kick.rs         # Kick, KickCombo
│       ├── combo.rs        # PunchKickCombo
│       └── aerial.rs       # JumpPunch, JumpKick
├── enemy/                   # Enemy AI code
│   └── components.rs       # Enemy, EnemyState
├── combat/                  # Combat system code
│   ├── components.rs       # Health, Hitbox, HurtBox, Stunned
│   └── messages.rs         # DamageEvent, EnemyDefeatedEvent, PlayerDefeatedEvent
└── common/                  # Shared utilities
    └── components.rs       # Direction, AnimationIndices, AnimationTimer
```

### ECS Pattern (Entity Component System)

This project uses Bevy's ECS architecture exclusively. All game logic is organized into:

- **Components**: Data structures attached to entities (organized by module)
  - Common: `Direction`, `AnimationIndices`, `AnimationTimer`
  - Player: `Player`, `PlayerState`, `JumpPhysics`, `ComboWindow`
  - Enemy: `Enemy`, `EnemyState`
  - Combat: `Health`, `Hitbox`, `HurtBox`, `Stunned`
- **Systems**: Pure functions that query components and operate on entities
- **Resources**: Global state (`GameState` tracks score, enemy count, spawn timing)
- **Messages**: Event-driven communication (`DamageEvent`, `EnemyDefeatedEvent`, `PlayerDefeatedEvent`)

**Important**: Bevy 0.17.3 renamed `Event` to `Message`. Use `#[derive(Message)]` and `MessageReader`/`MessageWriter` instead of `EventReader`/`EventWriter`.

### System Execution Order

Systems are strictly ordered using `.chain()` to prevent race conditions (src/main.rs):

1. **Input & State Management** (modular systems from `src/player/systems.rs`):
   - `player_input_system` - Delegates input to state's `handle_input()` method
   - `player_state_update_system` - Delegates transitions to state's `update()` method
   - `initialize_jump_physics` - Sets up jump velocity when entering Jump state
   - `player_sprite_update_system` - Updates sprites when state changes
2. **Movement**:
   - `player_physics_system` - Applies gravity/movement from state's `get_physics_config()`
   - `move_enemies` - Enemy AI movement
3. **Combat & Collision**:
   - `update_attack_hitboxes` → `detect_combat_collisions` → `detect_player_enemy_collisions`
4. **Damage Resolution**:
   - `handle_damage_events` → `update_stun_timers` → `handle_enemy_defeat` → `handle_player_defeat`
5. **Animation & Game Management**:
   - `animate_sprite`, `count_down`, `spawn_enemy`

This ordering ensures:
- State changes occur before sprite updates
- Hitboxes are positioned before collision detection
- Damage is applied before checking for defeats
- Movement happens with current frame's state

### Player State Machine (Modular Architecture)

The player state machine uses the **State Pattern** with 15 distinct states (each in its own file):

**Architecture**:
```rust
// Each state is a struct implementing StateLogic trait
pub trait StateLogic: Send + Sync + Clone {
    fn handle_input(&self, input: &InputContext) -> StateTransition;
    fn update(&self, ctx: &UpdateContext) -> StateTransition;
    fn get_animation_config(&self) -> AnimationConfig;
    fn get_physics_config(&self) -> PhysicsConfig;
    fn locks_input(&self) -> bool;
    fn is_attacking(&self) -> bool;
    fn get_damage(&self) -> i32;
}

// PlayerState enum holds state instances
pub enum PlayerState {
    Idle(IdleStateData),
    Run(RunStateData),
    Punch(PunchStateData),
    // ... all 15 states
}
```

**The 15 States**:

*Movement States* (src/player/states/idle.rs, movement.rs):
- `Idle` - Standing still
- `IdleToWalk`, `IdleToRun` - Transition animations
- `Walk`, `Run` - Ground movement

*Jump States* (src/player/states/jump.rs):
- `Jump` - Ascending phase (velocity > 0)
- `Fall` - Descending phase (velocity ≤ 0)
- `Land` - Landing animation

*Ground Combat States* (src/player/states/punch.rs, kick.rs, combo.rs):
- `Punch`, `PunchCombo` - Double-tap up arrow
- `Kick`, `KickCombo` - Double-tap down arrow
- `PunchKickCombo` - Punch combo → down arrow (mixed combo)

*Aerial Combat States* (src/player/states/aerial.rs):
- `JumpPunch`, `JumpKick` - Attacks while airborne

**State Encapsulation** - Each state defines its own:
- **Input Behavior**: What keyboard inputs trigger which transitions
- **Animation Config**: Sprite path, frame range, timing
- **Physics Config**: Movement speed, gravity, air control, input locks
- **Transition Logic**: When to auto-transition (animation complete, physics-driven)

**State Transition Flow**:
- Input-driven: State's `handle_input()` responds to keyboard and returns transition
- Animation-driven: State's `update()` checks if animation finished and returns transition
- Physics-driven: Jump states check velocity/ground position for transitions
- Combo window: 0.5 seconds to chain attacks
- Input locking: Movement disabled during attack animations via `locks_input()`

**Example: Punch State** (src/player/states/punch.rs):
```rust
impl StateLogic for PunchStateData {
    fn handle_input(&self, input: &InputContext) -> StateTransition {
        if input.up_arrow {
            StateTransition::To(PlayerStateType::PunchCombo) // Allow combo
        } else {
            StateTransition::None // Input locked during attack
        }
    }

    fn update(&self, ctx: &UpdateContext) -> StateTransition {
        if ctx.animation_finished {
            StateTransition::To(PlayerStateType::Idle) // Return to idle
        } else {
            StateTransition::None
        }
    }

    fn get_animation_config(&self) -> AnimationConfig {
        AnimationConfig {
            sprite_path: "player/punch-sheet.png",
            first_frame: 1,
            last_frame: 12,
            frame_duration: 0.03,
        }
    }

    fn get_physics_config(&self) -> PhysicsConfig {
        PhysicsConfig {
            ground_speed: 0.0,
            air_control: false,
            apply_gravity: false,
            locks_movement: true, // Cannot move during punch
        }
    }

    fn is_attacking(&self) -> bool { true }
    fn get_damage(&self) -> i32 { 2 }
}
```

**Adding a New State** (e.g., Dodge):
1. Create `src/player/states/dodge.rs`
2. Implement `StateLogic` trait for `DodgeStateData`
3. Add `Dodge(DodgeStateData)` variant to `PlayerState` enum
4. Add delegation cases to enum's methods
5. Add `Dodge` to `PlayerStateType` enum
6. Add sprite assets to `assets/player/dodge-sheet.png`
7. Done! Zero changes to systems or other states.

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
- `player_sprite_update_system` uses `Changed<PlayerState>` filter (src/player/systems.rs)
- Gets animation config from state's `get_animation_config()` method
- Creates new `TextureAtlasLayout` with correct column count for each animation
- **CRITICAL ORDER**: Resets atlas index to first frame BEFORE changing layout (prevents out-of-bounds access)
- Loads new sprite image AFTER atlas is configured (prevents blinking)

### Query Conflict Resolution

Player and enemy queries must be disjoint to avoid ECS conflicts:
- `player_physics_system`: `(With<Player>, Without<Enemy>)`
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
2. Verify state's `get_animation_config()` returns correct frame range
3. Ensure atlas index is reset before changing layout in `player_sprite_update_system`
4. Confirm texture atlas column count matches actual frames: `(last + 1) as u32`

### Debugging State Machine Issues

If state transitions don't work as expected:
1. Check the state's `handle_input()` method for input-driven transitions
2. Check the state's `update()` method for animation/physics-driven transitions
3. Verify `InputContext` and `UpdateContext` are populated correctly in systems
4. Use debug logging in state methods to trace transitions
5. Check that combo window timer is being ticked in `player_input_system`
6. For new states, ensure all delegation methods in `PlayerState` enum include the new variant
