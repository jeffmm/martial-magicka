# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Martial Magicka is a 2D side-scrolling martial arts game built with Bevy 0.17.3 (Rust game engine). The game features a player character with multiple combat states (punch, kick, combos) fighting against AI-controlled enemies that spawn periodically.

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

This project uses Bevy's ECS architecture. All game logic is organized into:

- **Components**: Data structures tagged onto entities (`PlayerState`, `EnemyState`, `Direction`, `AnimationIndices`, `AnimationTimer`)
- **Systems**: Functions that operate on entities with specific component combinations
- **Resources**: Global state (`GameState` tracks score, enemy count, spawn timing)

### Core Game Loop

The application is configured in `main()` (src/main.rs:6-21):
- `setup` system runs once at startup
- Update systems run every frame: `animate_sprite`, `keyboard_input_system`, `count_down`, `spawn_enemy`, `move_enemies`

### Player State Machine

Player has complex state system (src/main.rs:52-68) supporting:
- Movement: `Idle`, `IdleToWalk`, `IdleToRun`, `Walk`, `Run`, `Jump`, `Fall`, `Land`
- Combat: `Punch`, `PunchCombo`, `Kick`, `KickCombo`, `PunchKickCombo`, `JumpPunch`, `JumpKick`

Note: `update_player_sprite_from_state()` (src/main.rs:208-238) is defined but NOT currently called in the update loop. Player sprite updates are incomplete.

### Enemy AI

Enemies spawn every 2 seconds (max 6 active, see src/main.rs:136-171). AI behavior in `move_enemies()`:
- Enemies track player position and move toward them
- Move at 150 units/sec horizontally, 50 units/sec vertically
- Sprites flip based on direction relative to player

### Animation System

Sprite sheet animations use texture atlases:
- Player sprites: 640x640 tiles, 16 frames (assets/player/)
- Enemy sprites: 160x160 tiles, 12 frames (assets/enemies/)
- `AnimationTimer` controls frame advancement at 0.1s intervals
- `AnimationIndices` define first/last frame for each animation loop

### Asset Structure

```
assets/
├── desert.png          # Background sprite
├── player/
│   └── idle-sheet.png  # Player animation sprite sheets
└── enemies/
    └── ghost-sheet.png # Enemy animation sprite sheets
```

Player animations are loaded from `assets/player/` as sprite sheets. Enemy uses `ghost-sheet.png`.

## Development Notes

### Bevy Configuration

- Uses `dynamic_linking` feature for faster compile times in development
- Dev profile optimizes dependencies (opt-level 3) but minimal optimization for project code (opt-level 1)
- Edition 2024 Rust features enabled

### Input Handling

Keyboard controls in `keyboard_input_system()` (src/main.rs:241-271):
- A key: Move left
- D key: Move right
- Player moves at 150 units/sec

### Known Issues

- `update_player_sprite_from_state()` is defined but not hooked into the game loop, so player state transitions don't update sprites
- Player state changes need to trigger sprite sheet swaps for different animations (walk, run, punch, etc.)
