use macroquad::prelude::*;
use std::collections::{HashSet, VecDeque};

#[derive(Clone)]
pub struct Level {
    pub name: &'static str,
    pub subtitle: &'static str,
    pub map: Vec<Vec<char>>,
    pub start: Vec2,
    pub artifact: Vec2,
    pub exit: Vec2,
    pub sky: Color,
}

impl Level {
    fn new(
        name: &'static str,
        subtitle: &'static str,
        rows: &[&str],
        start: Vec2,
        artifact: Vec2,
        exit: Vec2,
        sky: Color,
    ) -> Self {
        assert!(!rows.is_empty(), "El mapa no puede estar vacio");
        let width = rows[0].chars().count();
        assert!(width > 2, "El mapa es demasiado angosto");
        assert!(
            rows.iter().all(|row| row.chars().count() == width),
            "Todas las filas del mapa deben tener el mismo ancho"
        );

        let level = Self {
            name,
            subtitle,
            map: rows.iter().map(|row| row.chars().collect()).collect(),
            start,
            artifact,
            exit,
            sky,
        };

        assert!(level.is_walkable(start.x, start.y), "Inicio dentro de pared");
        assert!(
            level.is_walkable(artifact.x, artifact.y),
            "Artefacto dentro de pared"
        );
        assert!(level.is_walkable(exit.x, exit.y), "Salida dentro de pared");
        level
    }

    pub fn width(&self) -> usize {
        self.map[0].len()
    }

    pub fn height(&self) -> usize {
        self.map.len()
    }

    /// Devuelve una pared si se consulta fuera del mapa. Esto evita accesos
    /// invalidos y hace que el borde siempre sea solido.
    pub fn tile_at(&self, x: i32, y: i32) -> char {
        if x < 0 || y < 0 || x >= self.width() as i32 || y >= self.height() as i32 {
            '#'
        } else {
            self.map[y as usize][x as usize]
        }
    }

    pub fn is_wall_tile(tile: char) -> bool {
        matches!(tile, '#' | 'B' | 'M' | 'T')
    }

    pub fn is_walkable(&self, x: f32, y: f32) -> bool {
        !Self::is_wall_tile(self.tile_at(x.floor() as i32, y.floor() as i32))
    }

    pub fn reachable(&self, from: Vec2, to: Vec2) -> bool {
        let start = (from.x.floor() as i32, from.y.floor() as i32);
        let goal = (to.x.floor() as i32, to.y.floor() as i32);
        let mut queue = VecDeque::from([start]);
        let mut visited = HashSet::from([start]);

        while let Some(cell) = queue.pop_front() {
            if cell == goal {
                return true;
            }
            for next in [
                (cell.0 + 1, cell.1),
                (cell.0 - 1, cell.1),
                (cell.0, cell.1 + 1),
                (cell.0, cell.1 - 1),
            ] {
                if !visited.contains(&next)
                    && !Self::is_wall_tile(self.tile_at(next.0, next.1))
                {
                    visited.insert(next);
                    queue.push_back(next);
                }
            }
        }
        false
    }
}

pub fn all_levels() -> Vec<Level> {
    let levels = vec![
        Level::new(
            "Distrito Neon",
            "Encuentra el nucleo y abre el portal",
            &[
                "########################",
                "#..........B...........#",
                "#.######...B...######..#",
                "#.#....#...B...........#",
                "#.#.##.#...BBBBBB......#",
                "#...##.........B.......#",
                "###.######.....B..MMMMM#",
                "#........#.....B..M....#",
                "#.BBBB...#........M....#",
                "#....B...######...M....#",
                "#....B............M....#",
                "#....BBBBB...TTTTTM....#",
                "#............T.........#",
                "#.MMMMMM.....T...#####.#",
                "#......M.....T.........#",
                "#......M.....TTTTT.....#",
                "#......................#",
                "########################",
            ],
            vec2(1.5, 1.5),
            vec2(20.5, 8.5),
            vec2(22.5, 16.5),
            Color::new(0.035, 0.02, 0.10, 1.0),
        ),
        Level::new(
            "Jardines de Cobalto",
            "Un laberinto organico bajo la luna",
            &[
                "########################",
                "#........M.............#",
                "#.MMMM...M...BBBBBBB...#",
                "#....M...M...B.........#",
                "#....M.......B...TTTT..#",
                "#....MMMMM...B...T.....#",
                "#............B...T.....#",
                "#.#########......T.....#",
                "#.#.......#......TTTTT.#",
                "#.#.MMMM..#####........#",
                "#.#....M........BBBBB..#",
                "#......M........B......#",
                "#.TTTTTM...######......#",
                "#.....T................#",
                "#.....T...MMMMMMMM.....#",
                "#.........M............#",
                "#......................#",
                "########################",
            ],
            vec2(2.5, 16.5),
            vec2(20.5, 2.5),
            vec2(22.5, 15.5),
            Color::new(0.01, 0.06, 0.11, 1.0),
        ),
        Level::new(
            "Boveda Solar",
            "Pasillos dorados de alta seguridad",
            &[
                "########################",
                "#......T...............#",
                "#.TTT..T..BBBBBBBB.....#",
                "#...T..T..B............#",
                "#...T.....B..MMMMMM....#",
                "#...TTTT..B..M.........#",
                "#..........B..M..#####.#",
                "#.BBBBBB......M........#",
                "#......B......MMMMM....#",
                "#......B..##########...#",
                "#.MMMMMB...............#",
                "#.M........TTTTTT......#",
                "#.M..#####......T......#",
                "#.M.............T......#",
                "#.MMMMM.........TTTT...#",
                "#......................#",
                "#......................#",
                "########################",
            ],
            vec2(1.5, 1.5),
            vec2(21.5, 7.5),
            vec2(22.5, 16.5),
            Color::new(0.11, 0.045, 0.01, 1.0),
        ),
    ];

    for level in &levels {
        assert!(level.reachable(level.start, level.artifact));
        assert!(level.reachable(level.artifact, level.exit));
    }
    levels
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_level_is_closed_and_solvable() {
        for level in all_levels() {
            for x in 0..level.width() {
                assert!(Level::is_wall_tile(level.map[0][x]));
                assert!(Level::is_wall_tile(level.map[level.height() - 1][x]));
            }
            for y in 0..level.height() {
                assert!(Level::is_wall_tile(level.map[y][0]));
                assert!(Level::is_wall_tile(level.map[y][level.width() - 1]));
            }
            assert!(level.reachable(level.start, level.artifact));
            assert!(level.reachable(level.artifact, level.exit));
        }
    }

    #[test]
    fn outside_the_map_is_always_a_wall() {
        let level = &all_levels()[0];
        assert_eq!(level.tile_at(-1, 2), '#');
        assert_eq!(level.tile_at(999, 2), '#');
    }
}

