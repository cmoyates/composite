<objective>
Extend the Level system with additional fields needed for the AI pathfinding system, and add utility functions.

This phase prepares the Level and utilities infrastructure needed by the AI module.
</objective>

<context>
**Previous Phase**: Phase 1 copied the bevy-advanced-cc project structure to composite.

Now we need to extend the Level system to support AI pathfinding. The pursue-ai-test project has:
- A `Level` Resource with additional fields: `grid_size`, `size`, `half_size`
- A `Polygon` struct with `is_container` field for identifying boundary polygons
- A `utils.rs` module with helper functions: `line_intersect`, `cross_product`, `side_of_line_detection`

**Key differences:**
- bevy-advanced-cc: `Polygon { points, collision_side, color, aabb }` - returned as Vec<Polygon>
- pursue-ai-test: `Level { polygons, grid_size, size, half_size }` Resource, `Polygon { points, color, is_container }`

**Files to examine:**
- `@pursue-ai-test/src/level.rs` - Level Resource struct, is_container logic
- `@pursue-ai-test/src/utils.rs` - Utility functions
- `./src/level.rs` - Current level system (from Phase 1)
</context>

<requirements>
1. **Create utils.rs**: Add a new utils module with helper functions from pursue-ai-test:
   - `line_intersect(line_1_start, line_1_end, line_2_start, line_2_end) -> Option<Vec2>`
   - `cross_product(a: Vec2, b: Vec2) -> f32`
   - `side_of_line_detection(line_start, line_end, point) -> f32`

2. **Extend Polygon struct**: Add `is_container: bool` field to identify boundary polygons

3. **Create Level Resource**: Wrap the polygons Vec in a Level Resource with:
   - `polygons: Vec<Polygon>`
   - `grid_size: f32`
   - `size: Vec2`
   - `half_size: Vec2`

4. **Update generate_level_polygons**: 
   - Return the Level Resource instead of just Vec<Polygon>
   - Calculate `is_container` using point-in-polygon test (polygon contains origin)
   - Return grid_size, size, half_size

5. **Update main.rs**:
   - Remove the inline Level Resource definition
   - Import Level from level.rs
   - Update s_init to use the new generate_level_polygons return value
   - Update s_render to use level.polygons

6. **Keep collision_side and aabb**: These are still needed for collision detection
</requirements>

<implementation>
The key changes are:

**level.rs modifications:**
```rust
#[derive(Resource)]
pub struct Level {
    pub polygons: Vec<Polygon>,
    pub grid_size: f32,
    pub size: Vec2,
    pub half_size: Vec2,
}

pub struct Polygon {
    pub points: Vec<Vec2>,
    pub collision_side: f32,
    pub color: Color,
    pub aabb: Aabb,
    pub is_container: bool,  // NEW
}
```

**Point-in-polygon logic from pursue-ai-test:**
```rust
fn point_in_polygon(polygon_lines: &[Vec2], point: Vec2) -> bool {
    // Cast ray and count intersections
}
```

**Important**: Keep the collision_side and aabb fields - they are used by the collision system. The is_container field is additional metadata for pathfinding.
</implementation>

<output>
Create/modify files with relative paths:
- `./src/utils.rs` - New utility functions module
- `./src/level.rs` - Extended Level Resource and Polygon struct
- `./src/main.rs` - Updated to use new Level Resource, add `mod utils;`
</output>

<verification>
Before declaring complete, verify your work:
1. Run `cargo check` to verify compilation
2. Run `cargo clippy --all-targets --all-features -D warnings` to check for warnings
3. Run `cargo run` to verify the game still works correctly
4. Player movement and collision detection should work exactly as before
</verification>

<success_criteria>
- utils.rs contains line_intersect, cross_product, side_of_line_detection functions
- Level is now a Resource with polygons, grid_size, size, half_size fields
- Polygon struct has is_container field
- generate_level_polygons returns the full Level Resource
- Game compiles and runs correctly with no behavior changes
- Collision system still works (uses collision_side and aabb)
</success_criteria>

