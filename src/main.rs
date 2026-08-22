mod audio;
mod level;
mod player;
mod raycaster;

use audio::GameAudio;
use level::{all_levels, Level};
use macroquad::prelude::*;
use player::Player;

#[derive(Clone, Copy, PartialEq)]
enum Screen {
    Menu,
    Playing,
    Victory,
}

struct Session {
    level_index: usize,
    player: Player,
    artifact_collected: bool,
    elapsed: f32,
    bump_cooldown: f32,
}

impl Session {
    fn new(level_index: usize, level: &Level) -> Self {
        Self {
            level_index,
            player: Player::new(level.start),
            artifact_collected: false,
            elapsed: 0.0,
            bump_cooldown: 0.0,
        }
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Neon Labyrinth - Rust Ray Caster".to_owned(),
        window_width: 1280,
        window_height: 720,
        high_dpi: true,
        fullscreen: false,
        sample_count: 1,
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let levels = all_levels();
    let mut selected_level = 0usize;
    let mut session = Session::new(selected_level, &levels[selected_level]);
    let mut screen = Screen::Menu;
    let mut sounds = GameAudio::load().await;
    sounds.start_music();

    loop {
        let dt = get_frame_time().clamp(0.0, 0.05);

        if is_key_pressed(KeyCode::M) {
            sounds.toggle_music();
        }

        match screen {
            Screen::Menu => {
                show_mouse(true);
                set_cursor_grab(false);
                selected_level = update_menu(selected_level, levels.len());
                draw_menu(&levels, selected_level, sounds.music_enabled);

                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                    session = Session::new(selected_level, &levels[selected_level]);
                    screen = Screen::Playing;
                    set_cursor_grab(true);
                    show_mouse(false);
                }
            }
            Screen::Playing => {
                if is_key_pressed(KeyCode::Escape) {
                    screen = Screen::Menu;
                    set_cursor_grab(false);
                    show_mouse(true);
                } else {
                    let level = &levels[session.level_index];
                    session.elapsed += dt;
                    session.bump_cooldown = (session.bump_cooldown - dt).max(0.0);

                    let blocked = session.player.update(level, dt);
                    if blocked && session.bump_cooldown <= 0.0 {
                        sounds.play_bump();
                        session.bump_cooldown = 0.18;
                    }

                    if !session.artifact_collected
                        && session.player.position.distance(level.artifact) < 0.48
                    {
                        session.artifact_collected = true;
                        sounds.play_collect();
                    }

                    if session.artifact_collected
                        && session.player.position.distance(level.exit) < 0.52
                    {
                        sounds.play_win();
                        screen = Screen::Victory;
                        set_cursor_grab(false);
                        show_mouse(true);
                    }

                    raycaster::draw_world(
                        level,
                        &session.player,
                        !session.artifact_collected,
                        get_time() as f32,
                    );
                    draw_hud(level, &session, sounds.music_enabled);
                    draw_minimap(level, &session);
                }
            }
            Screen::Victory => {
                set_cursor_grab(false);
                show_mouse(true);
                draw_victory(&levels[session.level_index], &session);

                if is_key_pressed(KeyCode::R) {
                    session = Session::new(session.level_index, &levels[session.level_index]);
                    screen = Screen::Playing;
                    set_cursor_grab(true);
                    show_mouse(false);
                } else if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Escape) {
                    screen = Screen::Menu;
                }
            }
        }

        next_frame().await;
    }
}

fn update_menu(mut selected: usize, count: usize) -> usize {
    if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) {
        selected = (selected + count - 1) % count;
    }
    if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) {
        selected = (selected + 1) % count;
    }
    if is_key_pressed(KeyCode::Key1) {
        selected = 0;
    }
    if is_key_pressed(KeyCode::Key2) && count > 1 {
        selected = 1;
    }
    if is_key_pressed(KeyCode::Key3) && count > 2 {
        selected = 2;
    }
    selected
}

fn draw_menu(levels: &[Level], selected: usize, music_enabled: bool) {
    let time = get_time() as f32;
    clear_background(Color::new(0.018, 0.012, 0.045, 1.0));
    draw_neon_grid(time);

    let w = screen_width();
    let h = screen_height();
    centered_text("NEON LABYRINTH", h * 0.17, 62.0, Color::new(0.55, 0.94, 1.0, 1.0));
    centered_text("RAY CASTER EN RUST", h * 0.23, 25.0, Color::new(0.94, 0.40, 1.0, 1.0));

    let card_w = (w * 0.25).clamp(220.0, 340.0);
    let gap = 18.0;
    let total = card_w * levels.len() as f32 + gap * (levels.len() - 1) as f32;
    let start_x = (w - total) * 0.5;
    let card_y = h * 0.33;

    for (index, level) in levels.iter().enumerate() {
        let x = start_x + index as f32 * (card_w + gap);
        let chosen = index == selected;
        let border = if chosen {
            Color::new(0.25, 0.92, 1.0, 1.0)
        } else {
            Color::new(0.30, 0.25, 0.46, 1.0)
        };
        let fill = if chosen {
            Color::new(0.08, 0.12, 0.22, 0.94)
        } else {
            Color::new(0.035, 0.035, 0.08, 0.90)
        };
        draw_rectangle(x, card_y, card_w, 150.0, fill);
        draw_rectangle_lines(x, card_y, card_w, 150.0, if chosen { 4.0 } else { 2.0 }, border);
        draw_text(&format!("0{}", index + 1), x + 18.0, card_y + 34.0, 26.0, border);
        draw_text(level.name, x + 18.0, card_y + 72.0, 25.0, WHITE);
        draw_wrapped_text(level.subtitle, x + 18.0, card_y + 101.0, card_w - 36.0, 18.0, GRAY);
    }

    centered_text("A / D o flechas: elegir     ENTER: jugar", h * 0.69, 23.0, WHITE);
    centered_text(
        "WASD: mover   |   Mouse/Q/E: girar   |   ESC: menu",
        h * 0.75,
        20.0,
        Color::new(0.65, 0.68, 0.78, 1.0),
    );
    centered_text(
        if music_enabled { "M: musica ON" } else { "M: musica OFF" },
        h * 0.82,
        18.0,
        Color::new(0.94, 0.40, 1.0, 1.0),
    );
}

fn draw_hud(level: &Level, session: &Session, music_enabled: bool) {
    let w = screen_width();
    let h = screen_height();
    let panel = Color::new(0.015, 0.02, 0.045, 0.80);
    draw_rectangle(18.0, 18.0, 285.0, 96.0, panel);
    draw_rectangle_lines(18.0, 18.0, 285.0, 96.0, 2.0, Color::new(0.18, 0.80, 0.95, 0.82));
    draw_text(level.name, 32.0, 47.0, 24.0, WHITE);
    draw_text(
        &format!("FPS: {}", get_fps()),
        32.0,
        75.0,
        21.0,
        Color::new(0.35, 1.0, 0.62, 1.0),
    );
    draw_text(
        if music_enabled { "MUSICA: ON" } else { "MUSICA: OFF" },
        142.0,
        75.0,
        18.0,
        Color::new(0.80, 0.55, 1.0, 1.0),
    );
    let objective = if session.artifact_collected {
        "OBJETIVO: llega al portal violeta"
    } else {
        "OBJETIVO: encuentra el nucleo azul"
    };
    draw_text(objective, 32.0, 101.0, 17.0, LIGHTGRAY);

    let cross = Color::new(0.80, 0.95, 1.0, 0.75);
    draw_line(w * 0.5 - 7.0, h * 0.5, w * 0.5 + 7.0, h * 0.5, 1.5, cross);
    draw_line(w * 0.5, h * 0.5 - 7.0, w * 0.5, h * 0.5 + 7.0, 1.5, cross);

    draw_text(
        "WASD mover  |  Mouse/Q/E girar  |  M musica  |  ESC menu",
        22.0,
        h - 22.0,
        18.0,
        Color::new(0.72, 0.75, 0.84, 0.90),
    );
}

fn draw_minimap(level: &Level, session: &Session) {
    let max_width = (screen_width() * 0.22).clamp(170.0, 245.0);
    let max_height = (screen_height() * 0.26).clamp(145.0, 195.0);
    let tile = (max_width / level.width() as f32)
        .min(max_height / level.height() as f32)
        .max(3.0);
    let map_width = tile * level.width() as f32;
    let map_height = tile * level.height() as f32;
    let origin = vec2(screen_width() - map_width - 24.0, 24.0);

    draw_rectangle(
        origin.x - 8.0,
        origin.y - 8.0,
        map_width + 16.0,
        map_height + 16.0,
        Color::new(0.005, 0.008, 0.02, 0.82),
    );
    draw_rectangle_lines(
        origin.x - 8.0,
        origin.y - 8.0,
        map_width + 16.0,
        map_height + 16.0,
        2.0,
        Color::new(0.25, 0.84, 1.0, 0.80),
    );

    for y in 0..level.height() {
        for x in 0..level.width() {
            let tile_char = level.map[y][x];
            if Level::is_wall_tile(tile_char) {
                let color = match tile_char {
                    'B' => Color::new(0.80, 0.15, 0.27, 0.88),
                    'M' => Color::new(0.10, 0.68, 0.39, 0.88),
                    'T' => Color::new(0.08, 0.62, 0.88, 0.88),
                    _ => Color::new(0.38, 0.31, 0.58, 0.88),
                };
                draw_rectangle(origin.x + x as f32 * tile, origin.y + y as f32 * tile, tile, tile, color);
            }
        }
    }

    let exit = origin + level.exit * tile;
    draw_circle(exit.x, exit.y, tile * 0.42, Color::new(0.92, 0.38, 1.0, 1.0));
    if !session.artifact_collected {
        let artifact = origin + level.artifact * tile;
        draw_circle(artifact.x, artifact.y, tile * 0.38, Color::new(0.25, 0.95, 1.0, 1.0));
    }

    let player = origin + session.player.position * tile;
    let direction = session.player.direction();
    let right = vec2(-direction.y, direction.x);
    draw_triangle(
        player + direction * tile * 0.75,
        player - direction * tile * 0.35 + right * tile * 0.42,
        player - direction * tile * 0.35 - right * tile * 0.42,
        YELLOW,
    );
}

fn draw_victory(level: &Level, session: &Session) {
    let time = get_time() as f32;
    clear_background(Color::new(0.015, 0.025, 0.07, 1.0));
    draw_neon_grid(-time * 0.35);
    let w = screen_width();
    let h = screen_height();
    let pulse = 0.85 + (time * 3.0).sin() * 0.15;

    for ring in 0..5 {
        draw_circle_lines(
            w * 0.5,
            h * 0.30,
            45.0 + ring as f32 * 17.0 + pulse * 6.0,
            3.0,
            Color::new(0.25, 0.90, 1.0, 0.60 - ring as f32 * 0.08),
        );
    }
    centered_text("MISION COMPLETADA", h * 0.53, 54.0, Color::new(0.55, 1.0, 0.88, 1.0));
    centered_text(
        &format!("Escapaste de {}", level.name),
        h * 0.61,
        28.0,
        WHITE,
    );
    centered_text(
        &format!("Tiempo: {:.1} segundos", session.elapsed),
        h * 0.67,
        22.0,
        Color::new(0.75, 0.78, 0.90, 1.0),
    );
    centered_text("R: repetir     ENTER: elegir otro nivel", h * 0.78, 22.0, WHITE);
}

fn draw_neon_grid(time: f32) {
    let w = screen_width();
    let h = screen_height();
    let horizon = h * 0.58;
    for index in 0..16 {
        let x = index as f32 / 15.0 * w;
        draw_line(w * 0.5, horizon, x, h, 1.0, Color::new(0.18, 0.35, 0.75, 0.30));
    }
    for index in 0..12 {
        let phase = ((index as f32 / 12.0 + time * 0.035).fract()).powf(2.0);
        let y = horizon + phase * (h - horizon);
        draw_line(0.0, y, w, y, 1.0, Color::new(0.35, 0.20, 0.80, 0.35));
    }
    for star in 0..42 {
        let x = ((star * 193) % 997) as f32 / 997.0 * w;
        let y = ((star * 71) % 397) as f32 / 397.0 * horizon;
        let glow = 0.45 + (time * 1.8 + star as f32).sin() * 0.30;
        draw_circle(x, y, 1.2, Color::new(0.55, 0.78, 1.0, glow));
    }
}

fn centered_text(text: &str, y: f32, size: f32, color: Color) {
    let dimensions = measure_text(text, None, size as u16, 1.0);
    draw_text(text, (screen_width() - dimensions.width) * 0.5, y, size, color);
}

fn draw_wrapped_text(text: &str, x: f32, y: f32, max_width: f32, size: f32, color: Color) {
    let mut line = String::new();
    let mut line_y = y;
    for word in text.split_whitespace() {
        let candidate = if line.is_empty() {
            word.to_owned()
        } else {
            format!("{} {}", line, word)
        };
        if measure_text(&candidate, None, size as u16, 1.0).width > max_width && !line.is_empty() {
            draw_text(&line, x, line_y, size, color);
            line = word.to_owned();
            line_y += size + 3.0;
        } else {
            line = candidate;
        }
    }
    if !line.is_empty() {
        draw_text(&line, x, line_y, size, color);
    }
}

