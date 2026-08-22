use crate::{level::Level, player::Player};
use macroquad::prelude::*;

const FOV_PLANE: f32 = 0.66;
const MAX_RAY_STEPS: usize = 128;

#[derive(Debug, Clone, Copy)]
pub struct RayHit {
    pub distance: f32,
    pub side: usize,
    pub tile: char,
    pub wall_x: f32,
}

pub fn cast_ray(level: &Level, origin: Vec2, ray_direction: Vec2) -> RayHit {
    let mut map_x = origin.x.floor() as i32;
    let mut map_y = origin.y.floor() as i32;

    let delta_x = if ray_direction.x.abs() < 0.000_001 {
        1.0e30
    } else {
        (1.0 / ray_direction.x).abs()
    };
    let delta_y = if ray_direction.y.abs() < 0.000_001 {
        1.0e30
    } else {
        (1.0 / ray_direction.y).abs()
    };

    let (step_x, mut side_x) = if ray_direction.x < 0.0 {
        (-1, (origin.x - map_x as f32) * delta_x)
    } else {
        (1, (map_x as f32 + 1.0 - origin.x) * delta_x)
    };
    let (step_y, mut side_y) = if ray_direction.y < 0.0 {
        (-1, (origin.y - map_y as f32) * delta_y)
    } else {
        (1, (map_y as f32 + 1.0 - origin.y) * delta_y)
    };

    let mut steps = 0;
    let (side, tile) = loop {
        steps += 1;
        let side = if side_x < side_y {
            side_x += delta_x;
            map_x += step_x;
            0
        } else {
            side_y += delta_y;
            map_y += step_y;
            1
        };
        let tile = level.tile_at(map_x, map_y);
        if Level::is_wall_tile(tile) || steps >= MAX_RAY_STEPS {
            break (side, tile);
        }
    };

    let distance = if side == 0 {
        (map_x as f32 - origin.x + (1 - step_x) as f32 * 0.5) / ray_direction.x
    } else {
        (map_y as f32 - origin.y + (1 - step_y) as f32 * 0.5) / ray_direction.y
    }
    .abs()
    .max(0.0001);

    let mut wall_x = if side == 0 {
        origin.y + distance * ray_direction.y
    } else {
        origin.x + distance * ray_direction.x
    };
    wall_x -= wall_x.floor();

    RayHit {
        distance,
        side,
        tile,
        wall_x,
    }
}

pub fn draw_world(level: &Level, player: &Player, artifact_visible: bool, time: f32) {
    let width = screen_width().max(1.0) as usize;
    let height = screen_height();
    draw_background(level, height);

    let direction = player.direction();
    let plane = vec2(-direction.y, direction.x) * FOV_PLANE;
    let mut depth_buffer = vec![1.0e30; width];

    for (x, depth) in depth_buffer.iter_mut().enumerate() {
        let camera_x = 2.0 * x as f32 / width as f32 - 1.0;
        let ray_direction = direction + plane * camera_x;
        let hit = cast_ray(level, player.position, ray_direction);
        *depth = hit.distance;
        draw_wall_column(x as f32, height, hit);
    }

    if artifact_visible {
        draw_billboard(
            level.artifact,
            player,
            plane,
            &depth_buffer,
            time,
            SpriteKind::Artifact,
        );
    }
    draw_billboard(
        level.exit,
        player,
        plane,
        &depth_buffer,
        time,
        SpriteKind::Exit,
    );

    draw_vignette();
}

fn draw_background(level: &Level, height: f32) {
    draw_rectangle(0.0, 0.0, screen_width(), height * 0.5, level.sky);
    draw_rectangle(
        0.0,
        height * 0.5,
        screen_width(),
        height * 0.5,
        Color::new(0.025, 0.028, 0.045, 1.0),
    );

    for index in 0..10 {
        let t = index as f32 / 10.0;
        let y = height * 0.5 + (t * t) * height * 0.5;
        draw_line(
            0.0,
            y,
            screen_width(),
            y,
            1.0,
            Color::new(0.10, 0.12, 0.18, 0.28),
        );
    }
}

fn draw_wall_column(x: f32, height: f32, hit: RayHit) {
    let wall_height = (height / hit.distance).min(height * 2.5);
    let top = (height - wall_height) * 0.5;
    let segments = 9;
    let segment_height = wall_height / segments as f32;

    for segment in 0..segments {
        let texture_y = segment as f32 / segments as f32;
        let color = textured_wall_color(hit.tile, hit.side, hit.wall_x, texture_y);
        draw_rectangle(
            x,
            top + segment as f32 * segment_height,
            1.15,
            segment_height + 0.6,
            color,
        );
    }
}

fn textured_wall_color(tile: char, side: usize, wall_x: f32, wall_y: f32) -> Color {
    let (mut r, mut g, mut b) = match tile {
        'B' => (0.82, 0.16, 0.27),
        'M' => (0.12, 0.68, 0.40),
        'T' => (0.08, 0.62, 0.88),
        _ => (0.43, 0.34, 0.68),
    };

    let brick_row = (wall_y * 9.0) as i32;
    let offset_x = if brick_row % 2 == 0 { wall_x } else { wall_x + 0.11 };
    let mortar = (wall_y * 9.0).fract() < 0.10 || (offset_x * 6.0).fract() < 0.055;
    if mortar {
        r *= 0.40;
        g *= 0.40;
        b *= 0.46;
    } else if tile == 'T' && (wall_x * 12.0).fract() > 0.88 {
        r = 0.62;
        g = 0.94;
        b = 1.0;
    } else if tile == 'M' && ((wall_x * 17.0 + wall_y * 13.0) as i32 % 5 == 0) {
        r *= 0.60;
        g *= 1.22;
        b *= 0.75;
    }

    let side_shade = if side == 1 { 0.72 } else { 1.0 };
    Color::new(r * side_shade, g * side_shade, b * side_shade, 1.0)
}

#[derive(Clone, Copy)]
enum SpriteKind {
    Artifact,
    Exit,
}

fn draw_billboard(
    position: Vec2,
    player: &Player,
    plane: Vec2,
    depth_buffer: &[f32],
    time: f32,
    kind: SpriteKind,
) {
    let relative = position - player.position;
    let direction = player.direction();
    let determinant = plane.x * direction.y - direction.x * plane.y;
    if determinant.abs() < 0.0001 {
        return;
    }

    let inverse = 1.0 / determinant;
    let transform_x = inverse * (direction.y * relative.x - direction.x * relative.y);
    let transform_y = inverse * (-plane.y * relative.x + plane.x * relative.y);
    if transform_y <= 0.15 {
        return;
    }

    let width = screen_width();
    let height = screen_height();
    let screen_x = (width * 0.5) * (1.0 + transform_x / transform_y);
    let center_index = screen_x.clamp(0.0, width - 1.0) as usize;
    if !(-width * 0.25..=width * 1.25).contains(&screen_x)
        || transform_y >= depth_buffer[center_index]
    {
        return;
    }

    let base_size = (height / transform_y).clamp(12.0, height * 0.78);
    match kind {
        SpriteKind::Artifact => draw_artifact(screen_x, height * 0.51, base_size, time),
        SpriteKind::Exit => draw_exit(screen_x, height * 0.50, base_size, time),
    }
}

/// Sprite animado por cuadros: cambia flotacion, alas, brillo y rotacion.
fn draw_artifact(x: f32, ground_y: f32, size: f32, time: f32) {
    let frame = ((time * 8.0) as i32).rem_euclid(4);
    let bob = [0.0, -0.045, 0.0, 0.045][frame as usize] * size;
    let wing = [0.26, 0.34, 0.29, 0.20][frame as usize] * size;
    let center_y = ground_y - size * 0.35 + bob;
    let glow = Color::new(0.20, 0.95, 1.0, 0.18);
    draw_circle(x, center_y, size * 0.30, glow);
    draw_triangle(
        vec2(x - size * 0.08, center_y),
        vec2(x - wing, center_y - size * 0.12),
        vec2(x - size * 0.17, center_y + size * 0.14),
        Color::new(0.18, 0.70, 0.95, 0.80),
    );
    draw_triangle(
        vec2(x + size * 0.08, center_y),
        vec2(x + wing, center_y - size * 0.12),
        vec2(x + size * 0.17, center_y + size * 0.14),
        Color::new(0.18, 0.70, 0.95, 0.80),
    );
    draw_poly(
        x,
        center_y,
        4,
        size * 0.15,
        time * 75.0,
        Color::new(0.75, 1.0, 1.0, 1.0),
    );
    draw_poly_lines(x, center_y, 4, size * 0.18, time * -55.0, 2.0, WHITE);
}

fn draw_exit(x: f32, center_y: f32, size: f32, time: f32) {
    let pulse = 0.92 + (time * 3.0).sin() * 0.08;
    let radius = size * 0.24 * pulse;
    draw_circle(x, center_y - size * 0.22, radius, Color::new(0.7, 0.2, 1.0, 0.13));
    draw_circle_lines(
        x,
        center_y - size * 0.22,
        radius,
        (size * 0.025).max(1.0),
        Color::new(0.92, 0.48, 1.0, 0.92),
    );
    draw_circle_lines(
        x,
        center_y - size * 0.22,
        radius * 0.68,
        (size * 0.012).max(1.0),
        Color::new(0.25, 0.88, 1.0, 0.84),
    );
}

fn draw_vignette() {
    let w = screen_width();
    let h = screen_height();
    let dark = Color::new(0.0, 0.0, 0.0, 0.30);
    draw_rectangle(0.0, 0.0, w, 14.0, dark);
    draw_rectangle(0.0, h - 14.0, w, 14.0, dark);
    draw_rectangle(0.0, 0.0, 18.0, h, dark);
    draw_rectangle(w - 18.0, 0.0, 18.0, h, dark);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::level::all_levels;

    #[test]
    fn ray_hits_a_wall_at_finite_distance() {
        let level = &all_levels()[0];
        let hit = cast_ray(level, level.start, vec2(1.0, 0.0));
        assert!(hit.distance.is_finite());
        assert!(hit.distance > 0.0);
        assert!(Level::is_wall_tile(hit.tile));
    }
}
