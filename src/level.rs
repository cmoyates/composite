use bevy::{color::Color, math::Vec2};
use rand::Rng;

/// Axis-aligned bounding box for spatial optimization
#[derive(Clone, Copy)]
pub struct Aabb {
    pub min: Vec2,
    pub max: Vec2,
}

impl Aabb {
    /// Create an AABB from a point and radius (for player collision checks)
    pub fn from_point_radius(center: Vec2, radius: f32) -> Self {
        Self {
            min: center - Vec2::splat(radius),
            max: center + Vec2::splat(radius),
        }
    }

    /// Check if this AABB overlaps with another AABB
    pub fn overlaps(&self, other: &Aabb) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
    }

    /// Expand AABB by a given amount in all directions
    pub fn expand(&self, amount: f32) -> Self {
        Self {
            min: self.min - Vec2::splat(amount),
            max: self.max + Vec2::splat(amount),
        }
    }
}

pub struct Polygon {
    pub points: Vec<Vec2>,
    pub collision_side: f32,
    pub color: Color,
    /// Cached bounding box for spatial optimization
    pub aabb: Aabb,
}

const LEVEL_DATA: &[u8] = include_bytes!("../assets/level.json");

pub fn generate_level_polygons(grid_size: f32) -> Vec<Polygon> {
    let mut rng = rand::rng();

    let res = std::str::from_utf8(LEVEL_DATA);
    let json_data: Vec<Vec<u32>> = serde_json::from_str(res.unwrap()).unwrap();

    let offset = Vec2::new(
        json_data[0].len() as f32 * -grid_size / 2.0,
        json_data.len() as f32 * grid_size / 2.0,
    );

    let mut line_points: Vec<Vec2> = Vec::new();

    for y in 0..json_data.len() {
        for x in 0..json_data[y].len() {
            let tile = json_data[y][x];

            match tile {
                1 => {
                    // Squares

                    // Left edge
                    if x == 0 || json_data[y][x - 1] == 0 {
                        line_points.push(Vec2::new(x as f32 * grid_size, y as f32 * grid_size));
                        line_points
                            .push(Vec2::new(x as f32 * grid_size, (y + 1) as f32 * grid_size));
                    }
                    // Right edge
                    if x == json_data[y].len() - 1 || json_data[y][x + 1] == 0 {
                        line_points
                            .push(Vec2::new((x + 1) as f32 * grid_size, y as f32 * grid_size));
                        line_points.push(Vec2::new(
                            (x + 1) as f32 * grid_size,
                            (y + 1) as f32 * grid_size,
                        ));
                    }
                    // Top edge
                    if y == 0 || json_data[y - 1][x] == 0 {
                        line_points.push(Vec2::new(x as f32 * grid_size, y as f32 * grid_size));
                        line_points
                            .push(Vec2::new((x + 1) as f32 * grid_size, y as f32 * grid_size));
                    }
                    // Bottom edge
                    if y == json_data.len() - 1 || json_data[y + 1][x] == 0 {
                        line_points
                            .push(Vec2::new(x as f32 * grid_size, (y + 1) as f32 * grid_size));
                        line_points.push(Vec2::new(
                            (x + 1) as f32 * grid_size,
                            (y + 1) as f32 * grid_size,
                        ));
                    }
                }
                2..=5 => {
                    // Right triangles

                    let triangle_type = tile - 2;

                    match triangle_type {
                        0 => {
                            // Bottom left

                            // Hypotenuse
                            line_points.push(Vec2::new(x as f32 * grid_size, y as f32 * grid_size));
                            line_points.push(Vec2::new(
                                (x + 1) as f32 * grid_size,
                                (y + 1) as f32 * grid_size,
                            ));

                            // Bottom edge
                            if y == json_data.len() - 1 || json_data[y + 1][x] == 0 {
                                line_points.push(Vec2::new(
                                    x as f32 * grid_size,
                                    (y + 1) as f32 * grid_size,
                                ));
                                line_points.push(Vec2::new(
                                    (x + 1) as f32 * grid_size,
                                    (y + 1) as f32 * grid_size,
                                ));
                            }

                            // Left edge
                            if x == 0 || json_data[y][x - 1] == 0 {
                                line_points
                                    .push(Vec2::new(x as f32 * grid_size, y as f32 * grid_size));
                                line_points.push(Vec2::new(
                                    x as f32 * grid_size,
                                    (y + 1) as f32 * grid_size,
                                ));
                            }
                        }
                        1 => {
                            // Bottom right

                            // Hypotenuse
                            line_points
                                .push(Vec2::new((x + 1) as f32 * grid_size, y as f32 * grid_size));
                            line_points
                                .push(Vec2::new(x as f32 * grid_size, (y + 1) as f32 * grid_size));

                            // Bottom edge
                            if y == json_data.len() - 1 || json_data[y + 1][x] == 0 {
                                line_points.push(Vec2::new(
                                    x as f32 * grid_size,
                                    (y + 1) as f32 * grid_size,
                                ));
                                line_points.push(Vec2::new(
                                    (x + 1) as f32 * grid_size,
                                    (y + 1) as f32 * grid_size,
                                ));
                            }

                            // Right edge
                            if x == json_data[y].len() - 1 || json_data[y][x + 1] == 0 {
                                line_points.push(Vec2::new(
                                    (x + 1) as f32 * grid_size,
                                    y as f32 * grid_size,
                                ));
                                line_points.push(Vec2::new(
                                    (x + 1) as f32 * grid_size,
                                    (y + 1) as f32 * grid_size,
                                ));
                            }
                        }
                        2 => {
                            // Top left

                            // Hypotenuse
                            line_points
                                .push(Vec2::new(x as f32 * grid_size, (y + 1) as f32 * grid_size));
                            line_points
                                .push(Vec2::new((x + 1) as f32 * grid_size, y as f32 * grid_size));

                            // Top edge
                            if y == 0 || json_data[y - 1][x] == 0 {
                                line_points
                                    .push(Vec2::new(x as f32 * grid_size, y as f32 * grid_size));
                                line_points.push(Vec2::new(
                                    (x + 1) as f32 * grid_size,
                                    y as f32 * grid_size,
                                ));
                            }

                            // Left edge
                            if x == 0 || json_data[y][x - 1] == 0 {
                                line_points
                                    .push(Vec2::new(x as f32 * grid_size, y as f32 * grid_size));
                                line_points.push(Vec2::new(
                                    x as f32 * grid_size,
                                    (y + 1) as f32 * grid_size,
                                ));
                            }
                        }
                        3 => {
                            // Top right

                            // Hypotenuse
                            line_points.push(Vec2::new(
                                (x + 1) as f32 * grid_size,
                                (y + 1) as f32 * grid_size,
                            ));
                            line_points.push(Vec2::new(x as f32 * grid_size, y as f32 * grid_size));

                            // Top edge
                            if y == 0 || json_data[y - 1][x] == 0 {
                                line_points
                                    .push(Vec2::new(x as f32 * grid_size, y as f32 * grid_size));
                                line_points.push(Vec2::new(
                                    (x + 1) as f32 * grid_size,
                                    y as f32 * grid_size,
                                ));
                            }

                            // Right edge
                            if x == json_data[y].len() - 1 || json_data[y][x + 1] == 0 {
                                line_points.push(Vec2::new(
                                    (x + 1) as f32 * grid_size,
                                    y as f32 * grid_size,
                                ));
                                line_points.push(Vec2::new(
                                    (x + 1) as f32 * grid_size,
                                    (y + 1) as f32 * grid_size,
                                ));
                            }
                        }
                        _ => {}
                    }
                }
                6..=9 => {
                    // Isosceles triangles (not implemented)
                }
                _ => {}
            }
        }
    }

    let mut line_count = line_points.len() / 2;

    // Remove superfluous points

    let mut point_removal_data = Some(((0, 0), (0, 0)));

    // While there are points to remove
    while point_removal_data.is_some() {
        point_removal_data = None;

        'outer: for i in 0..line_count {
            for j in 0..line_count {
                // If the lines are the same, skip
                if i == j {
                    continue;
                }

                // Check if either of the points are shared

                let line_1_start = line_points[i * 2];
                let line_1_end = line_points[i * 2 + 1];

                let line_2_start = line_points[j * 2];
                let line_2_end = line_points[j * 2 + 1];

                let mut shared_point: Option<(usize, usize)> = None;
                let mut unique_points: Option<(usize, usize)> = None;

                if line_1_start == line_2_start {
                    shared_point = Some((i * 2, j * 2));
                    unique_points = Some((i * 2 + 1, j * 2 + 1));
                } else if line_1_start == line_2_end {
                    shared_point = Some((i * 2, j * 2 + 1));
                    unique_points = Some((i * 2 + 1, j * 2));
                } else if line_1_end == line_2_start {
                    shared_point = Some((i * 2 + 1, j * 2));
                    unique_points = Some((i * 2, j * 2 + 1));
                } else if line_1_end == line_2_end {
                    shared_point = Some((i * 2 + 1, j * 2 + 1));
                    unique_points = Some((i * 2, j * 2));
                }

                // If there is no shared point, skip
                if shared_point.is_none() {
                    continue;
                }

                // Check if the lines are parallel

                let dot = (line_1_start - line_1_end)
                    .normalize()
                    .dot((line_2_start - line_2_end).normalize());
                if dot.abs() == 1.0 {
                    // if so flag the point for removal and break out of the outer for loop
                    point_removal_data = Some((shared_point.unwrap(), unique_points.unwrap()));
                    break 'outer;
                }
            }
        }

        // If there is a point to remove
        if let Some(point_removal_data) = point_removal_data {
            // Store the unique vertices
            let unique_vert_1 = line_points[point_removal_data.1 .0];
            let unique_vert_2 = line_points[point_removal_data.1 .1];

            // Remove the shared and unique vertices
            let mut removal_indices = vec![
                point_removal_data.0 .0,
                point_removal_data.0 .1,
                point_removal_data.1 .0,
                point_removal_data.1 .1,
            ];
            removal_indices.sort();
            removal_indices.reverse();
            for i in removal_indices {
                line_points.remove(i);
            }

            // Add the unique vertices back
            line_points.push(unique_vert_1);
            line_points.push(unique_vert_2);

            // Update the line count
            line_count -= 1;
        }
    }

    for point in &mut line_points {
        point.x += offset.x;
        point.y *= -1.0;
        point.y += offset.y;
    }

    // Separate the lines into polygons
    let mut polygons: Vec<Polygon> = Vec::new();

    // While there are lines left
    while line_count > 0 {
        // Create a new polygon
        let mut polygon_lines: Vec<Vec2> = Vec::new();

        // Add the first line to the polygon
        polygon_lines.push(line_points[0]);
        polygon_lines.push(line_points[1]);

        // Remove the first line from the list of lines
        line_points.remove(0);
        line_points.remove(0);

        // Decrement the line count
        line_count -= 1;

        let start_vert = polygon_lines[0];
        let mut current_vert = polygon_lines[polygon_lines.len() - 1];

        // While the polygon is not closed
        while start_vert != current_vert {
            // Find the next line that connects to current_vert
            let mut found_idx = None;
            let mut connects_at_start = false;

            for i in 0..line_count {
                let line_start = line_points[i * 2];
                let line_end = line_points[i * 2 + 1];

                if line_start == current_vert {
                    found_idx = Some(i);
                    connects_at_start = true;
                    break;
                } else if line_end == current_vert {
                    found_idx = Some(i);
                    connects_at_start = false;
                    break;
                }
            }

            if let Some(i) = found_idx {
                let line_start = line_points[i * 2];
                let line_end = line_points[i * 2 + 1];

                if connects_at_start {
                    // Add the line to the polygon
                    polygon_lines.push(line_end);
                    // Set the current vertex to the end of the line
                    current_vert = line_end;
                } else {
                    // Add the line to the polygon
                    polygon_lines.push(line_start);
                    // Set the current vertex to the start of the line
                    current_vert = line_start;
                }

                // Remove the line from the list of lines
                line_points.remove(i * 2);
                line_points.remove(i * 2);

                // Decrement the line count
                line_count -= 1;
            }
        }

        let collision_side = calculate_winding_order(&polygon_lines).signum();

        let color = Color::srgb(
            rng.random_range(0.0..=1.0),
            rng.random_range(0.0..=1.0),
            rng.random_range(0.0..=1.0),
        );

        // Compute bounding box for spatial optimization
        let aabb = compute_polygon_aabb(&polygon_lines);

        // Add the polygon to the list of polygons
        polygons.push(Polygon {
            points: polygon_lines,
            collision_side,
            color,
            aabb,
        });
    }

    polygons
}

fn calculate_winding_order(vertices: &[Vec2]) -> f32 {
    let mut sum = 0.0;

    for i in 0..vertices.len() {
        let p1 = vertices[i];
        let p2 = vertices[(i + 1) % vertices.len()];
        sum += (p2.x - p1.x) * (p2.y + p1.y);
    }

    sum
}

/// Compute axis-aligned bounding box for a polygon
fn compute_polygon_aabb(points: &[Vec2]) -> Aabb {
    if points.is_empty() {
        return Aabb {
            min: Vec2::ZERO,
            max: Vec2::ZERO,
        };
    }

    let mut min_x = points[0].x;
    let mut min_y = points[0].y;
    let mut max_x = points[0].x;
    let mut max_y = points[0].y;

    for point in points.iter().skip(1) {
        min_x = min_x.min(point.x);
        min_y = min_y.min(point.y);
        max_x = max_x.max(point.x);
        max_y = max_y.max(point.y);
    }

    Aabb {
        min: Vec2::new(min_x, min_y),
        max: Vec2::new(max_x, max_y),
    }
}

