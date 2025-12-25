mod ai;
mod collisions;
mod level;
mod utils;

use ::bevy::prelude::*;
use bevy::{app::AppExit, input::ButtonInput, window::PresentMode};
use ai::{
    pathfinding::{init_pathfinding_graph, PathfindingPlugin},
    platformer_ai::{AIPhysics, PlatformerAI, PlatformerAIPlugin},
    pursue_ai::{PursueAI, PursueAIState, PursueAIPlugin, PURSUE_AI_AGENT_RADIUS},
};
use collisions::{s_collision, s_debug_collision, CollisionPlugin};
use level::{generate_level_polygons, Level};

// Floating point comparison epsilon
const EPSILON: f32 = 1e-6;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
        .insert_resource(InputDir { dir: Vec2::ZERO })
        .insert_resource(ShouldExit(false))
        .insert_resource(GizmosVisible { visible: false })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Advanced Character Controller".to_string(),
                present_mode: PresentMode::AutoNoVsync,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(CollisionPlugin)
        .add_plugins(PathfindingPlugin)
        .add_plugins(PlatformerAIPlugin)
        .add_plugins(PursueAIPlugin)
        // Startup systems
        .add_systems(Startup, s_init)
        // Update systems
        .add_systems(Update, s_input)
        .add_systems(Update, s_handle_gizmo_toggle)
        .add_systems(Update, s_movement.after(s_input))
        .add_systems(Update, s_timers.after(s_collision))
        .add_systems(Update, s_debug_collision.after(s_collision))
        .add_systems(Update, s_render.after(s_timers))
        // Exit system runs last to ensure clean shutdown
        .add_systems(Update, s_exit.after(s_render))
        .run();
}

#[derive(Resource)]
pub struct InputDir {
    pub dir: Vec2,
}

#[derive(Resource)]
pub struct ShouldExit(bool);

#[derive(Resource)]
pub struct GizmosVisible {
    pub visible: bool,
}

// Movement constants (units: pixels/second)
// Converted from 5.0 pixels/frame at 60fps = 300.0 pixels/second
pub const PLAYER_MAX_SPEED: f32 = 300.0;

// Acceleration scalers (units: 1/second)
// These control how quickly velocity approaches target velocity
// First value: acceleration rate when input is active (1/second)
// Second value: deceleration rate when input is inactive (1/second)
// Converted from frame-based: 0.2 per frame at 60fps = 12.0 per second
pub const PLAYER_ACCELERATION_SCALERS: (f32, f32) = (12.0, 24.0);

// Timer constants (units: seconds)
// These represent the duration windows for jump buffering, coyote time, and wall contact
// Originally 10 frames at 60fps = 0.166 seconds
pub const MAX_JUMP_TIMER: f32 = 0.166;
pub const MAX_GROUNDED_TIMER: f32 = 0.166;
pub const MAX_WALLED_TIMER: f32 = 0.166;

// Physics constants
// Velocity constants (units: pixels/second)
// Converted from frame-based: multiply by 60 (frames/second)
pub const JUMP_VELOCITY: f32 = 540.0; // 9.0 pixels/frame * 60
pub const WALL_JUMP_VELOCITY_Y: f32 = 270.0; // 4.5 pixels/frame * 60
pub const WALL_JUMP_VELOCITY_X: f32 = 468.0; // 7.8 pixels/frame * 60

// Gravity constant (units: pixels/second²)
// Converted from frame-based: 0.5 pixels/frame² at 60fps = 1800.0 pixels/second²
pub const GRAVITY_STRENGTH: f32 = 1800.0;

// Wall jump acceleration reduction (unitless multiplier)
pub const WALL_JUMP_ACCELERATION_REDUCTION: f32 = 0.5;

// Jump release velocity divisor (unitless)
pub const JUMP_RELEASE_VELOCITY_DIVISOR: f32 = 3.0;

// Collision detection thresholds
// NORMAL_DOT_THRESHOLD: Minimum dot product for considering a surface a "wall" (0.8 ≈ 37°)
pub const NORMAL_DOT_THRESHOLD: f32 = 0.8;
// GROUND_NORMAL_Y_THRESHOLD: Minimum Y component of normal to be considered "ground"
pub const GROUND_NORMAL_Y_THRESHOLD: f32 = 0.01;
// CEILING_NORMAL_Y_THRESHOLD: Maximum Y component of normal to be considered "ceiling"
pub const CEILING_NORMAL_Y_THRESHOLD: f32 = -0.01;

/// Player component: Contains gameplay state (timers, jump state, wall contact)
#[derive(Component)]
pub struct Player {
    /// Jump buffer timer: Time remaining (seconds) to execute a buffered jump input
    jump_timer: f32,
    /// Coyote time timer: Time remaining (seconds) player can still jump after leaving ground
    grounded_timer: f32,
    /// Wall contact timer: Time remaining (seconds) player is considered touching a wall
    wall_timer: f32,
    /// Wall direction: X direction of wall contact (-1.0 for left, 1.0 for right, 0.0 for none)
    wall_direction: f32,
    /// Whether player has performed a wall jump (prevents multiple wall jumps)
    has_wall_jumped: bool,
    /// Whether player is currently grounded (derived from grounded_timer > 0)
    is_grounded: bool,
    /// Last wall normal vector (for wall jump direction calculation)
    last_wall_normal: Option<Vec2>,
}

/// Physics component: Contains pure physics state (position, velocity, acceleration, collision)
#[derive(Component)]
pub struct Physics {
    /// Previous frame's position (for collision detection)
    pub prev_position: Vec2,
    /// Current velocity vector (pixels/second)
    pub velocity: Vec2,
    /// Current acceleration vector (pixels/second²)
    pub acceleration: Vec2,
    /// Collision radius (pixels)
    pub radius: f32,
    /// Surface normal at current position (zero if not touching surface)
    pub normal: Vec2,
}

/// Initial setup system
pub fn s_init(mut commands: Commands, pathfinding: ResMut<ai::pathfinding::PathfindingGraph>) {
    // Spawn camera
    commands.spawn((Camera2d, Transform::default()));

    // Spawn player
    let initial_position = Vec3::new(0.0, -50.0, 0.0);
    commands.spawn((
        Transform::from_translation(initial_position),
        Physics {
            prev_position: initial_position.xy(),
            velocity: Vec2::ZERO,
            acceleration: Vec2::ZERO,
            radius: 12.0,
            normal: Vec2::ZERO,
        },
        Player {
            jump_timer: 0.0,
            grounded_timer: 0.0,
            wall_timer: 0.0,
            wall_direction: 0.0,
            has_wall_jumped: false,
            is_grounded: false,
            last_wall_normal: None,
        },
    ));

    // Spawn AI agent
    let ai_initial_position = Vec3::new(0.0, -250.0, 0.0);
    commands.spawn((
        Transform::from_translation(ai_initial_position),
        AIPhysics {
            prev_position: ai_initial_position.xy(),
            velocity: Vec2::ZERO,
            acceleration: Vec2::ZERO,
            radius: PURSUE_AI_AGENT_RADIUS,
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

    // Init level
    {
        let grid_size = 32.0;

        let level = generate_level_polygons(grid_size);

        // Initialize pathfinding graph
        init_pathfinding_graph(&level, pathfinding);

        commands.insert_resource(level);
    }
}

/// Input system
pub fn s_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut should_exit: ResMut<ShouldExit>,
    mut input_dir: ResMut<InputDir>,
    mut player_query: Query<(&mut Player, &mut Physics)>,
) {
    // Escape to exit - set flag for dedicated exit system to handle
    if keyboard_input.just_pressed(KeyCode::Escape) {
        should_exit.0 = true;
        return;
    }

    if let Ok((mut player_data, mut player_physics)) = player_query.single_mut() {
        let mut direction = Vec2::ZERO;

        // Arrow keys to move
        if keyboard_input.pressed(KeyCode::ArrowUp) {
            direction.y += 1.0;
        }
        if keyboard_input.pressed(KeyCode::ArrowDown) {
            direction.y -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::ArrowLeft) {
            direction.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::ArrowRight) {
            direction.x += 1.0;
        }

        // Space to jump
        if keyboard_input.just_pressed(KeyCode::Space) {
            player_data.jump_timer = MAX_JUMP_TIMER;
        }

        // Variable jump height: reduce velocity if jump key released early
        if keyboard_input.just_released(KeyCode::Space) && player_physics.velocity.y > EPSILON {
            player_physics.velocity.y /= JUMP_RELEASE_VELOCITY_DIVISOR;
        }

        // Normalize direction
        direction = direction.normalize_or_zero();

        // Set direction resource
        input_dir.dir = direction;
    }
}

/// Movement system
/// Implements frame-rate independent physics using delta time and semi-implicit Euler integration
pub fn s_movement(
    mut player_query: Query<(&mut Transform, &mut Physics, &mut Player)>,
    input_dir: Res<InputDir>,
    time: Res<Time>,
) {
    if let Ok((mut player_transform, mut player_physics, mut player_data)) =
        player_query.single_mut()
    {
        // Clamp delta time to prevent huge jumps on first frame or frame skips
        // Maximum delta time of 1/30th second (30 FPS minimum)
        let dt = time.delta_secs().min(1.0 / 30.0);

        // Use epsilon comparison for floating point values
        let player_falling = player_physics.normal.length_squared() < EPSILON;
        let no_input = input_dir.dir.length_squared() < EPSILON;

        // Rotate input according to the normal (compute locally, don't mutate resource)
        let mut effective_input_dir = input_dir.dir;
        if !no_input
            && !player_falling
            && input_dir.dir.dot(player_physics.normal).abs() < NORMAL_DOT_THRESHOLD
        {
            let mut new_input_dir = Vec2::new(player_physics.normal.y, -player_physics.normal.x);

            if new_input_dir.dot(input_dir.dir) < 0.0 {
                new_input_dir *= -1.0;
            }

            effective_input_dir = new_input_dir;
        }

        // If the player is on a wall and is trying to move away from it
        let player_move_off_wall = player_physics.normal.x.abs() >= NORMAL_DOT_THRESHOLD
            && effective_input_dir.x.abs() >= NORMAL_DOT_THRESHOLD
            && player_physics.normal.x.signum() != effective_input_dir.x.signum();

        // Calculate acceleration (units: pixels/second²)
        {
            // Apply acceleration towards target velocity
            // This creates smooth acceleration/deceleration
            player_physics.acceleration = (effective_input_dir * PLAYER_MAX_SPEED
                - player_physics.velocity)
                * if no_input {
                    // Deceleration
                    PLAYER_ACCELERATION_SCALERS.1
                } else {
                    // Acceleration
                    PLAYER_ACCELERATION_SCALERS.0
                };

            // Wall jump physics - reduce acceleration after wall jump
            player_physics.acceleration *= if player_data.has_wall_jumped {
                WALL_JUMP_ACCELERATION_REDUCTION
            } else {
                1.0
            };

            // If the player is falling
            if player_falling {
                // Ignore any other acceleration in the y direction
                player_physics.acceleration.y = 0.0;
            }
            // Unless the player is on a wall and is trying to move away from it
            if !player_move_off_wall {
                // Remove the acceleration in the direction of the normal
                // This prevents acceleration into walls
                let acceleration_adjustment =
                    player_physics.normal * player_physics.acceleration.dot(player_physics.normal);
                player_physics.acceleration -= acceleration_adjustment;
            }
        }

        // Apply gravity directly to velocity (not additive to acceleration)
        // Gravity is a force that should be applied consistently each frame
        {
            if player_move_off_wall || player_falling {
                // Gravity goes down (negative Y)
                player_physics.velocity.y -= GRAVITY_STRENGTH * dt;
            } else {
                // Gravity goes towards the normal (for wall/ceiling walking)
                let gravity_normal_dir = player_physics.normal * GRAVITY_STRENGTH * dt;
                player_physics.velocity += gravity_normal_dir;
            }
        }

        // Jumping
        {
            // If the player is trying to jump
            if player_data.jump_timer > 0.0 {
                // If on the ground
                if player_data.grounded_timer > 0.0 {
                    // Jump
                    player_physics.velocity.y = JUMP_VELOCITY;
                    player_data.jump_timer = 0.0;
                    player_data.grounded_timer = 0.0;
                }
                // If on a wall
                else if player_data.wall_timer > 0.0 {
                    // Wall jump
                    player_physics.velocity.y = WALL_JUMP_VELOCITY_Y;
                    player_physics.velocity.x = player_data.wall_direction * WALL_JUMP_VELOCITY_X;
                    player_data.jump_timer = 0.0;
                    player_data.wall_timer = 0.0;
                    player_data.wall_direction = 0.0;
                    player_data.has_wall_jumped = true;
                }
            }
        }

        // Update physics using semi-implicit Euler integration
        // 1. Update velocity: v(t+dt) = v(t) + a(t) * dt
        // 2. Update position: x(t+dt) = x(t) + v(t+dt) * dt
        // This is more stable than explicit Euler and preserves energy better
        player_physics.prev_position = player_transform.translation.xy();

        // Apply acceleration to velocity (scaled by delta time)
        let acceleration_dt = player_physics.acceleration * dt;
        player_physics.velocity += acceleration_dt;

        // Update position using new velocity (scaled by delta time)
        let velocity_dt = player_physics.velocity * dt;
        player_transform.translation.x += velocity_dt.x;
        player_transform.translation.y += velocity_dt.y;
    }
}

/// Render system
pub fn s_render(
    mut gizmos: Gizmos,
    player_query: Query<(&Transform, &Physics), With<Player>>,
    ai_query: Query<(&Transform, &AIPhysics), With<PursueAI>>,
    level: Res<Level>,
) {
    // Draw level
    for polygon in &level.polygons {
        gizmos.linestrip_2d(polygon.points.iter().copied(), polygon.color);
    }

    // Draw player
    if let Ok((player_transform, player_physics)) = player_query.single() {
        gizmos.circle_2d(
            player_transform.translation.xy(),
            player_physics.radius,
            Color::WHITE,
        );
    }

    // Draw AI agents
    for (ai_transform, ai_physics) in ai_query.iter() {
        gizmos.circle_2d(
            ai_transform.translation.xy(),
            ai_physics.radius,
            Color::srgb(1.0, 0.0, 0.0), // Red for AI
        );
    }
}

/// Timer system: Decrements all timers by delta time
pub fn s_timers(time: Res<Time>, mut player_query: Query<&mut Player>) {
    if let Ok(mut player_data) = player_query.single_mut() {
        let dt = time.delta_secs();

        if player_data.jump_timer > 0.0 {
            player_data.jump_timer -= dt;
            if player_data.jump_timer < 0.0 {
                player_data.jump_timer = 0.0;
            }
        }

        if player_data.grounded_timer > 0.0 {
            player_data.grounded_timer -= dt;
            if player_data.grounded_timer < 0.0 {
                player_data.grounded_timer = 0.0;
                player_data.is_grounded = false;
            } else {
                player_data.is_grounded = true;
            }
        } else {
            player_data.is_grounded = false;
        }

        if player_data.wall_timer > 0.0 {
            player_data.wall_timer -= dt;
            if player_data.wall_timer < 0.0 {
                player_data.wall_timer = 0.0;
                player_data.wall_direction = 0.0;
            }
        }
    }
}

/// Gizmo toggle system: Toggles debug gizmo visibility with G key
pub fn s_handle_gizmo_toggle(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut gizmos_visible: ResMut<GizmosVisible>,
) {
    // G to toggle gizmos
    if keyboard_input.just_pressed(KeyCode::KeyG) {
        gizmos_visible.visible = !gizmos_visible.visible;
    }
}

/// Exit system: Handles clean application exit after all other systems complete
/// This runs last in the update loop to ensure no race conditions with other systems
pub fn s_exit(should_exit: Res<ShouldExit>, mut exit: MessageWriter<AppExit>) {
    if should_exit.0 {
        exit.write(AppExit::Success);
    }
}
