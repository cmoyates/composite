<objective>
Port the AI module from pursue-ai-test to composite, adapting the physics to use frame-rate independent constants.

This phase adds the full AI system: pathfinding graph, A* algorithm, platformer AI movement, and pursue AI state machine.
</objective>

<context>
**Previous Phases**: 
- Phase 1: Copied bevy-advanced-cc foundation
- Phase 2: Extended Level with is_container, added utils.rs

Now we add the AI module from pursue-ai-test. The key challenge is that pursue-ai-test uses **frame-based physics** while composite uses **frame-rate independent physics**.

**Physics constant conversion:**
- pursue-ai-test: `GRAVITY_STRENGTH = 0.5` (pixels/frame, ~30 pixels/second² at 60fps)
- composite: `GRAVITY_STRENGTH = 1800.0` (pixels/second²)

**Files to port from pursue-ai-test:**
- `src/ai/mod.rs` - Module root
- `src/ai/a_star.rs` - A* pathfinding algorithm
- `src/ai/pathfinding.rs` - Pathfinding graph and plugin
- `src/ai/platformer_ai.rs` - Platformer movement system
- `src/ai/pursue_ai/mod.rs` - State machine root
- `src/ai/pursue_ai/movement.rs` - Movement utilities
- `src/ai/pursue_ai/wander.rs` - Wander state
</context>

<requirements>
1. **Create AI module structure**: 
   ```
   src/ai/
   ├── mod.rs
   ├── a_star.rs
   ├── pathfinding.rs
   ├── platformer_ai.rs
   └── pursue_ai/
       ├── mod.rs
       ├── movement.rs
       └── wander.rs
   ```

2. **Port pathfinding.rs**: 
   - Copy PathfindingPlugin, PathfindingGraph, PathfindingGraphNode, etc.
   - Update imports to use composite's Level and utils
   - **CRITICAL**: Convert GRAVITY_STRENGTH references:
     - pursue-ai-test uses `crate::GRAVITY_STRENGTH` (0.5)
     - Change to use composite's constant (1800.0) from main.rs

3. **Port a_star.rs**: 
   - Copy find_path, AStarNode, PathNode
   - Update imports

4. **Port platformer_ai.rs**:
   - Copy PlatformerAI component and plugin
   - **CRITICAL**: Convert physics constants to frame-rate independent:
     - Original: `WANDER_MAX_SPEED = 3.0` → New: `WANDER_MAX_SPEED = 180.0` (3.0 * 60)
     - Original: `PLATFORMER_AI_JUMP_FORCE = 8.0` → New: `PLATFORMER_AI_JUMP_FORCE = 480.0` (8.0 * 60)
     - Original: `ACCELERATION_SCALERS = (0.2, 0.4)` → New: `ACCELERATION_SCALERS = (12.0, 24.0)` (per second)
   - Use `time.delta_secs()` for physics calculations

5. **Port pursue_ai/ directory**:
   - Copy mod.rs, movement.rs, wander.rs
   - Update imports and physics conversions

6. **Create AIPhysics component**: The AI needs its own Physics-like component since the player's Physics has player-specific fields. Create:
   ```rust
   #[derive(Component)]
   pub struct AIPhysics {
       pub prev_position: Vec2,
       pub velocity: Vec2,
       pub acceleration: Vec2,
       pub radius: f32,
       pub normal: Vec2,
       pub grounded: bool,
       pub walled: i8,
       pub has_wall_jumped: bool,
   }
   ```

7. **Update main.rs**:
   - Add `mod ai;`
   - Register PathfindingPlugin, PlatformerAIPlugin, PursueAIPlugin
   - Initialize pathfinding graph in s_init

8. **Add AI collision support**: 
   - Create a separate collision system for AI entities or extend s_collision
   - AI entities need collision detection with the level
</requirements>

<implementation>
**Physics Conversion Approach:**

The original pursue-ai-test applies physics per-frame without delta time. To convert to frame-rate independent:

1. Velocities: Multiply by 60 (assuming 60fps target)
2. Accelerations: Multiply by 60²  
3. In systems: Multiply by `time.delta_secs()` when applying

**Example conversion in platformer_ai.rs:**
```rust
// Original (frame-based):
physics.velocity += physics.acceleration;
transform.translation += physics.velocity.extend(0.0);

// Converted (time-based):
let dt = time.delta_secs();
physics.velocity += physics.acceleration * dt;
transform.translation += (physics.velocity * dt).extend(0.0);
```

**Pathfinding gravity conversion:**
The pathfinding jumpability_check uses gravity for trajectory calculation. Update to use the new GRAVITY_STRENGTH constant.

**Important**: The AI should be functional but won't pursue anything yet - that comes in Phase 4.
</implementation>

<output>
Create/modify files:
- `./src/ai/mod.rs` - AI module exports
- `./src/ai/a_star.rs` - A* pathfinding
- `./src/ai/pathfinding.rs` - Pathfinding graph
- `./src/ai/platformer_ai.rs` - Platformer AI movement
- `./src/ai/pursue_ai/mod.rs` - Pursue AI state machine
- `./src/ai/pursue_ai/movement.rs` - Movement utilities
- `./src/ai/pursue_ai/wander.rs` - Wander state
- `./src/main.rs` - Register AI plugins
- `./src/collisions.rs` - Add AI collision support
</output>

<verification>
Before declaring complete:
1. Run `cargo check` to verify compilation
2. Run `cargo clippy --all-targets --all-features -D warnings`
3. Run `cargo run` - verify:
   - Game still works
   - Pathfinding graph initializes (check for "Spatial index built" message)
   - No runtime panics
</verification>

<success_criteria>
- All AI module files created with correct structure
- Physics constants converted to frame-rate independent values
- AI systems use time.delta_secs() for physics
- PathfindingPlugin, PlatformerAIPlugin, PursueAIPlugin registered
- Pathfinding graph initializes from level
- Project compiles and runs without errors
- Player controller still works correctly
</success_criteria>

