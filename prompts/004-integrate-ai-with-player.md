<objective>
Integrate the AI to pursue the player entity instead of a static GoalPoint.

This final phase makes the AI actively chase the player, creating a predator-prey dynamic.
</objective>

<context>
**Previous Phases**:
- Phase 1: Copied bevy-advanced-cc foundation
- Phase 2: Extended Level, added utils
- Phase 3: Added AI module with pathfinding and pursue AI

Currently the pursue-ai-test project uses a `GoalPoint` component that the user moves with arrow keys. We need to make the AI pursue the `Player` entity instead.

**Key changes needed:**
1. The AI needs to query for the Player's position instead of GoalPoint
2. The pathfinding goal should be the player's current position
3. The AI should spawn in the level and start pursuing
</context>

<requirements>
1. **Spawn AI Entity**: In s_init, spawn an AI entity with:
   - Transform (different starting position from player)
   - AIPhysics component
   - PlatformerAI component
   - PursueAI component with state: Wander (will transition to Pursue when it "sees" player)

2. **Update PlatformerAI to track Player**:
   - Modify s_platformer_ai_movement to query for the Player's position
   - Use player position as the goal for pathfinding
   - Remove GoalPoint references

3. **Update PursueAI state transitions**:
   - Wander: Random exploration until player is "seen"
   - Pursue: Active chase using pathfinding to player
   - For now, can simplify: Always pursue (skip Wander) or implement simple distance-based state switch

4. **Add AI collision system**:
   - Create s_ai_collision similar to s_collision but for AIPhysics entities
   - Or extend s_collision to handle both Player and AI

5. **Update rendering**:
   - Draw AI entity as a different color circle (red for predator)
   - Draw player as white (already done)
   - Optionally: Draw pathfinding debug info when pressing G key

6. **Add player tracking**:
   - Create a marker component `PlayerTarget` or use a resource to store player position
   - AI systems query this to know where to pathfind
</requirements>

<implementation>
**Entity spawning in s_init:**
```rust
// Spawn AI
commands.spawn((
    Transform::from_translation(Vec3::new(-100.0, -50.0, 0.0)),  // Different from player
    AIPhysics {
        prev_position: Vec2::new(-100.0, -50.0),
        velocity: Vec2::ZERO,
        acceleration: Vec2::ZERO,
        radius: 8.0,  // PURSUE_AI_AGENT_RADIUS
        normal: Vec2::ZERO,
        grounded: false,
        walled: 0,
        has_wall_jumped: false,
    },
    PlatformerAI {
        current_target_node: None,
        jump_from_pos: None,
        jump_to_pos: None,
        cached_path: None,
        last_goal_position: None,
        current_path_index: 0,
    },
    PursueAI {
        state: PursueAIState::Pursue,  // Start in Pursue mode
        current_wander_goal: None,
    },
));
```

**Goal acquisition pattern:**
```rust
// In s_platformer_ai_movement:
fn s_platformer_ai_movement(
    mut ai_query: Query<(&mut Transform, &mut AIPhysics, &mut PlatformerAI), With<PursueAI>>,
    player_query: Query<&Transform, With<Player>>,
    pathfinding: Res<PathfindingGraph>,
    time: Res<Time>,
) {
    let player_pos = if let Ok(player_transform) = player_query.single() {
        player_transform.translation.xy()
    } else {
        return;
    };

    for (mut transform, mut physics, mut ai) in ai_query.iter_mut() {
        // Use player_pos as goal for pathfinding
        // ...
    }
}
```

**System ordering:**
```rust
// Ensure AI systems run in correct order
.add_systems(Update, s_platformer_ai_movement.after(s_input))
.add_systems(Update, s_ai_collision.after(s_platformer_ai_movement))
```
</implementation>

<output>
Modify files:
- `./src/main.rs` - Spawn AI entity, update system registration
- `./src/ai/platformer_ai.rs` - Query Player instead of GoalPoint
- `./src/ai/pursue_ai/mod.rs` - Update state machine to use Player
- `./src/collisions.rs` - Add AI collision handling
</output>

<verification>
Before declaring complete:
1. Run `cargo check` and `cargo clippy --all-targets --all-features -D warnings`
2. Run `cargo run` and verify:
   - Player spawns (white circle)
   - AI spawns (red circle) at different position
   - AI moves toward player using pathfinding
   - AI can jump between platforms
   - Both player and AI have collision detection
   - Player can still control their character with arrow keys + space
   - Press Escape to exit cleanly
</verification>

<success_criteria>
- AI entity spawns in the level
- AI actively pursues player position
- Pathfinding works - AI navigates around obstacles, jumps between platforms
- AI collision detection works - AI doesn't fall through floors
- Player controls still work
- Visual distinction between player (white) and AI (red)
- Game is playable: player tries to evade AI predator
</success_criteria>

