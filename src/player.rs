use crate::level::Level;
use macroquad::prelude::*;

const MOVE_SPEED: f32 = 3.1;
const KEYBOARD_TURN_SPEED: f32 = 2.25;
const MOUSE_SENSITIVITY: f32 = 2.35;
const COLLISION_RADIUS: f32 = 0.20;

pub struct Player {
    pub position: Vec2,
    pub angle: f32,
}

impl Player {
    pub fn new(position: Vec2) -> Self {
        Self {
            position,
            angle: 0.0,
        }
    }

    pub fn direction(&self) -> Vec2 {
        vec2(self.angle.cos(), self.angle.sin())
    }

    /// Actualiza el movimiento y devuelve true si se intento atravesar una pared.
    pub fn update(&mut self, level: &Level, dt: f32) -> bool {
        let mut turn = 0.0;
        if is_key_down(KeyCode::Left) || is_key_down(KeyCode::Q) {
            turn -= 1.0;
        }
        if is_key_down(KeyCode::Right) || is_key_down(KeyCode::E) {
            turn += 1.0;
        }
        self.angle += turn * KEYBOARD_TURN_SPEED * dt;

        // Macroquad entrega el delta normalizado. Se limita para ignorar saltos
        // grandes cuando la ventana recupera el foco.
        let mouse_delta = mouse_delta_position().x.clamp(-0.25, 0.25);
        self.angle -= mouse_delta * MOUSE_SENSITIVITY;

        let forward = self.direction();
        let right = vec2(-forward.y, forward.x);
        let mut wish = Vec2::ZERO;

        if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
            wish += forward;
        }
        if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
            wish -= forward;
        }
        if is_key_down(KeyCode::A) {
            wish -= right;
        }
        if is_key_down(KeyCode::D) {
            wish += right;
        }

        if wish.length_squared() <= f32::EPSILON {
            return false;
        }

        let delta = wish.normalize() * MOVE_SPEED * dt;
        let mut blocked = false;

        // Resolver cada eje por separado permite deslizarse junto a las paredes.
        let candidate_x = vec2(self.position.x + delta.x, self.position.y);
        if can_stand(level, candidate_x) {
            self.position.x = candidate_x.x;
        } else {
            blocked = true;
        }

        let candidate_y = vec2(self.position.x, self.position.y + delta.y);
        if can_stand(level, candidate_y) {
            self.position.y = candidate_y.y;
        } else {
            blocked = true;
        }

        blocked
    }
}

fn can_stand(level: &Level, position: Vec2) -> bool {
    for offset in [
        vec2(-COLLISION_RADIUS, -COLLISION_RADIUS),
        vec2(COLLISION_RADIUS, -COLLISION_RADIUS),
        vec2(-COLLISION_RADIUS, COLLISION_RADIUS),
        vec2(COLLISION_RADIUS, COLLISION_RADIUS),
    ] {
        let point = position + offset;
        if !level.is_walkable(point.x, point.y) {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::level::all_levels;

    #[test]
    fn walls_reject_the_player_circle() {
        let level = &all_levels()[0];
        assert!(!can_stand(level, vec2(0.5, 0.5)));
        assert!(can_stand(level, level.start));
    }
}
