use bevy::{
    app::{App, Plugin, Update},
    color::Color,
    ecs::{
        schedule::IntoScheduleConfigs,
        system::{Query, Res},
    },
    gizmos::gizmos::Gizmos,
    math::{Vec2, Vec3Swizzles},
    transform::components::Transform,
};

use crate::{
    level::{Aabb, Level},
    s_movement, Physics, Player, CEILING_NORMAL_Y_THRESHOLD,
    GROUND_NORMAL_Y_THRESHOLD, MAX_GROUNDED_TIMER, MAX_WALLED_TIMER, NORMAL_DOT_THRESHOLD,
};

// Collision detection constants
const RAYCAST_DIRECTION_SCALE: f32 = 10000.0;
const RAYCAST_DIRECTION: Vec2 = Vec2::new(2.0, 1.0);
const TOUCH_THRESHOLD: f32 = 0.5;
const DEBUG_NORMAL_LINE_LENGTH: f32 = 12.0;
const DISTANCE_CALCULATION_RADIUS_MULTIPLIER: f32 = 2.0;

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, s_collision.after(s_movement));
    }
}

pub fn s_collision(
    mut player_query: Query<(&mut Transform, &mut Physics, &mut Player)>,
    level: Res<Level>,
) {
    if let Ok((mut player_transform, mut player_physics, mut player_data)) =
        player_query.single_mut()
    {
        let mut adjustment = Vec2::ZERO;
        let mut new_player_normal = Vec2::ZERO;

        // Pre-compute player AABB for broad-phase collision detection
        let player_pos = player_transform.translation.xy();
        let player_aabb = Aabb::from_point_radius(player_pos, player_physics.radius);
        // Expand AABB slightly to account for movement
        let expanded_player_aabb = player_aabb.expand(player_physics.radius * 0.5);

        // Pre-compute radius squared to avoid repeated calculations
        let radius_sq = player_physics.radius.powi(2);
        let touch_threshold_sq = (player_physics.radius + TOUCH_THRESHOLD).powi(2);

        for polygon in &level.polygons {
            // Broad-phase: AABB pre-check to skip polygons far from player
            if !expanded_player_aabb.overlaps(&polygon.aabb) {
                continue;
            }

            let mut intersect_counter = 0;
            let mut colliding_with_polygon = false;

            // Raycast intersection check for point-in-polygon test
            for i in 1..polygon.points.len() {
                let start = polygon.points[i - 1];
                let end = polygon.points[i];

                let intersection = line_intersect(
                    start,
                    end,
                    player_pos,
                    player_pos + RAYCAST_DIRECTION * RAYCAST_DIRECTION_SCALE,
                );

                if intersection.is_some() {
                    intersect_counter += 1;
                }
            }

            // Narrow-phase: detailed collision detection with polygon edges
            for i in 1..polygon.points.len() {
                let start = polygon.points[i - 1];
                let end = polygon.points[i];

                let previous_side_of_line =
                    side_of_line_detection(start, end, player_physics.prev_position);

                if previous_side_of_line != polygon.collision_side {
                    continue;
                }

                let (distance_sq, projection) =
                    find_projection(start, end, player_pos, player_physics.radius);

                let colliding_with_line = distance_sq <= radius_sq;
                colliding_with_polygon = colliding_with_polygon || colliding_with_line;

                let touching_line = distance_sq <= touch_threshold_sq;

                if touching_line {
                    let normal_dir = (player_pos - projection).normalize_or_zero();

                    // If the line is not above the player
                    if normal_dir.y >= CEILING_NORMAL_Y_THRESHOLD {
                        // Add the normal dir to the players new normal
                        new_player_normal -= normal_dir;

                        // If the player is on a wall
                        if normal_dir.x.abs() >= NORMAL_DOT_THRESHOLD {
                            player_data.wall_timer = MAX_WALLED_TIMER;
                            player_data.wall_direction = normal_dir.x.signum();
                            player_data.last_wall_normal = Some(normal_dir);
                            player_data.has_wall_jumped = false;
                        }

                        // If the player is on the ground
                        if normal_dir.y > GROUND_NORMAL_Y_THRESHOLD {
                            player_data.grounded_timer = MAX_GROUNDED_TIMER;
                            player_data.is_grounded = true;
                            player_data.wall_timer = 0.0;
                            player_data.wall_direction = 0.0;
                            player_data.has_wall_jumped = false;
                        }
                    }
                }

                if colliding_with_line {
                    let mut delta = (player_pos - projection).normalize_or_zero();

                    if delta.y < CEILING_NORMAL_Y_THRESHOLD {
                        player_physics.velocity.y = 0.0;
                    }

                    // Use squared distance calculation, only compute sqrt when needed
                    let distance = distance_sq.sqrt();
                    delta *= player_physics.radius - distance;

                    if delta.x.abs() > adjustment.x.abs() {
                        adjustment.x = delta.x;
                    }
                    if delta.y.abs() > adjustment.y.abs() {
                        adjustment.y = delta.y;
                    }
                }
            }

            // Point-in-polygon check: if inside polygon and raycast intersects odd number of times
            if colliding_with_polygon && intersect_counter % 2 == 1 {
                player_transform.translation = player_physics.prev_position.extend(0.0);
            }
        }

        // Update the players normal
        new_player_normal = new_player_normal.normalize_or_zero();
        player_physics.normal = new_player_normal;

        // Remove the players velocity in the direction of the normal
        let velocity_adjustment =
            player_physics.velocity.dot(new_player_normal) * new_player_normal;

        player_physics.velocity -= velocity_adjustment;

        // Update the players position
        player_transform.translation += adjustment.extend(0.0);
    }
}

/// Debug rendering system for collision visualization (optional, runs after collision)
pub fn s_debug_collision(
    player_query: Query<(&Transform, &Physics, &Player)>,
    level: Res<Level>,
    mut gizmos: Gizmos,
) {
    if let Ok((player_transform, player_physics, _player_data)) = player_query.single() {
        let player_pos = player_transform.translation.xy();
        let touch_threshold_sq = (player_physics.radius + TOUCH_THRESHOLD).powi(2);

        // Pre-compute player AABB for broad-phase
        let player_aabb = Aabb::from_point_radius(player_pos, player_physics.radius);
        let expanded_player_aabb = player_aabb.expand(player_physics.radius * 0.5);

        for polygon in &level.polygons {
            // Skip polygons far from player
            if !expanded_player_aabb.overlaps(&polygon.aabb) {
                continue;
            }

            // Draw collision normals for touching surfaces
            for i in 1..polygon.points.len() {
                let start = polygon.points[i - 1];
                let end = polygon.points[i];

                let (distance_sq, projection) =
                    find_projection(start, end, player_pos, player_physics.radius);

                let touching_line = distance_sq <= touch_threshold_sq;

                if touching_line {
                    let normal_dir = (player_pos - projection).normalize_or_zero();

                    // If the line is not above the player
                    if normal_dir.y >= CEILING_NORMAL_Y_THRESHOLD {
                        gizmos.line_2d(
                            player_pos,
                            player_pos - normal_dir * DEBUG_NORMAL_LINE_LENGTH,
                            Color::WHITE,
                        );
                    }
                }
            }
        }
    }
}

pub fn find_projection(start: Vec2, end: Vec2, point: Vec2, radius: f32) -> (f32, Vec2) {
    let point_vec = point - start;
    let line_vec = end - start;

    let line_vec_normalized = line_vec.normalize();

    let dot = point_vec.dot(line_vec_normalized);

    let projection_point = line_vec_normalized * dot + start;

    if dot < 0.0 {
        return (
            point_vec.length_squared() + radius * DISTANCE_CALCULATION_RADIUS_MULTIPLIER,
            projection_point,
        );
    }

    if dot.powi(2) > (end - start).length_squared() {
        return (
            (point - end).length_squared() + radius * DISTANCE_CALCULATION_RADIUS_MULTIPLIER,
            projection_point,
        );
    }

    let dist = (point - projection_point).length_squared();

    (dist, projection_point)
}

pub fn side_of_line_detection(line_start: Vec2, line_end: Vec2, point: Vec2) -> f32 {
    let determinant = (line_end.x - line_start.x) * (point.y - line_start.y)
        - (line_end.y - line_start.y) * (point.x - line_start.x);

    determinant.signum()
}

pub fn line_intersect(
    line_1_start: Vec2,
    line_1_end: Vec2,
    line_2_start: Vec2,
    line_2_end: Vec2,
) -> Option<Vec2> {
    let line_1 = line_1_end - line_1_start;
    let line_2 = line_2_end - line_2_start;
    let r_cross_s = cross_product(line_1, line_2);
    let a_to_c = line_2_start - line_1_start;
    let t = cross_product(a_to_c, line_2) / r_cross_s;
    let u = cross_product(a_to_c, line_1) / r_cross_s;

    if (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u) {
        Some(Vec2::new(
            line_1_start.x + t * line_1.x,
            line_1_start.y + t * line_1.y,
        ))
    } else {
        None
    }
}

pub fn cross_product(a: Vec2, b: Vec2) -> f32 {
    a.x * b.y - a.y * b.x
}

