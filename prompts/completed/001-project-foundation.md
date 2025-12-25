<objective>
Set up the composite project foundation by copying the bevy-advanced-cc structure as the base.

This phase establishes the core player controller and physics system that will serve as the foundation for adding AI agents later.
</objective>

<context>
We are merging two Bevy 0.17.3 game projects:
- `bevy-advanced-cc`: Advanced 2D character controller with frame-rate independent physics
- `pursue-ai-test`: AI pathfinding and pursuit system

The composite project currently only has a "Hello, World!" main.rs. We need to:

1. Use bevy-advanced-cc as the foundation
2. Later add AI from pursue-ai-test

**Key files to examine:**

- `@bevy-advanced-cc/Cargo.toml` - Dependencies to copy
- `@bevy-advanced-cc/src/main.rs` - Core game structure
- `@bevy-advanced-cc/src/collisions.rs` - Collision system
- `@bevy-advanced-cc/src/level.rs` - Level generation
- `@bevy-advanced-cc/assets/level.json` - Level data
  </context>

<requirements>
1. **Update Cargo.toml**: Copy all dependencies from bevy-advanced-cc:
   - bevy = "0.17.3"
   - rand = "0.9"
   - serde = { version = "1.0", features = ["derive"] }
   - serde_json = "1.0"

2. **Copy Source Files**: Port the following from bevy-advanced-cc to composite/src/:

   - `main.rs` - Full copy with all systems, components, resources, constants
   - `collisions.rs` - Full copy with CollisionPlugin
   - `level.rs` - Full copy with Polygon struct, Aabb struct, generate_level_polygons

3. **Copy Assets**: Create composite/assets/ and copy level.json from bevy-advanced-cc/assets/

4. **Verify Build**: The project should compile and run with `cargo run`
   </requirements>

<implementation>
Copy files exactly as they are from bevy-advanced-cc. Do not modify the physics or collision logic - we need the frame-rate independent physics intact.

**File structure to create:**

```
composite/
├── Cargo.toml (updated)
├── assets/
│   └── level.json (copied)
└── src/
    ├── main.rs (copied)
    ├── collisions.rs (copied)
    └── level.rs (copied)
```

</implementation>

<output>
Create/modify files with relative paths:
- `./Cargo.toml` - Updated with all dependencies
- `./assets/level.json` - Copied from bevy-advanced-cc
- `./src/main.rs` - Copied from bevy-advanced-cc
- `./src/collisions.rs` - Copied from bevy-advanced-cc  
- `./src/level.rs` - Copied from bevy-advanced-cc
</output>

<verification>
Before declaring complete, verify your work:
1. Run `cargo check` to verify the project compiles
2. Run `cargo run` to verify the game launches and displays the player + level
3. Verify the player can move with arrow keys and jump with space
4. Press Escape to exit cleanly
</verification>

<success_criteria>

- All source files copied correctly from bevy-advanced-cc
- Cargo.toml has all required dependencies
- Project compiles without errors or warnings
- Game runs and player controls work correctly
- Level geometry renders and collision detection works
  </success_criteria>
