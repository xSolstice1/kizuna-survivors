use macroquad::prelude::*;
use sim::world::DT;
use sim::{Input, RunState, Stress, World, WorldConfig};

mod chest_ui;
mod render;
mod sprites;
mod stage;
mod ui;

use render::{Assets, Fx};

fn window_conf() -> Conf {
    Conf { window_title: "Kizuna Survivors".into(), window_width: 1280, window_height: 720, ..Default::default() }
}

#[derive(PartialEq)]
enum Screen {
    Title,
    Playing,
    Paused,
}

fn read_input() -> Input {
    let mut d = sim::Vec2::ZERO;
    if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
        d.y -= 1.0;
    }
    if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
        d.y += 1.0;
    }
    if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
        d.x -= 1.0;
    }
    if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
        d.x += 1.0;
    }
    Input { move_dir: d, ..Default::default() }
}

fn new_world(stress: Option<Stress>) -> World {
    let seed = (macroquad::miniquad::date::now() * 1000.0) as u64;
    World::new(WorldConfig { seed, stress, ..Default::default() })
}

#[macroquad::main(window_conf)]
async fn main() {
    let assets = Assets::load();
    let stage_art = stage::StageArt::load();
    let mut fx = Fx::new();
    let mut screen = Screen::Title;
    let mut stress: Option<Stress> = None;
    let mut world = new_world(stress);
    let mut acc = 0.0f32;
    let mut sim_ms = 0.0f32;
    let mut chest_anim: Option<chest_ui::ChestAnim> = None;
    // Space is latched until the sim next steps; at high refresh rates many frames take no step.
    let mut special_queued = false;

    loop {
        let dt = get_frame_time().min(0.1);
        let now = get_time();

        if screen == Screen::Title {
            ui::title(now);
            if is_mouse_button_pressed(MouseButton::Left) || is_key_pressed(KeyCode::Enter) {
                world = new_world(stress);
                fx.clear();
                screen = Screen::Playing;
            }
            next_frame().await;
            continue;
        }

        // --- Global keys ---
        let over = matches!(world.state, RunState::Dead | RunState::Won);
        // Esc pauses only during play; on the shrine screen it means "walk away".
        if is_key_pressed(KeyCode::Escape) && !over && (screen == Screen::Paused || !world.paused()) {
            screen = if screen == Screen::Paused { Screen::Playing } else { Screen::Paused };
        }
        if is_key_pressed(KeyCode::N) {
            fx.show_numbers = !fx.show_numbers;
        }
        let toggle_stress = is_key_pressed(KeyCode::F1);
        if toggle_stress {
            stress = match stress {
                None => Some(Stress { enemies: 2000, projectiles: 4000 }),
                Some(_) => None,
            };
        }
        let restart = (is_key_pressed(KeyCode::R) && (screen == Screen::Paused || over))
            || (over && is_mouse_button_pressed(MouseButton::Left))
            || toggle_stress;
        if restart {
            world = new_world(stress);
            fx.clear();
            screen = Screen::Playing;
        }
        // Dev shortcuts (remove before release): F2 skip to the mid-boss, F3 max loadout, F4 open a chest,
        // F5 open a legendary chest, F6 every library weapon (ignores hero pool/slots), F7 shrine underfoot,
        // F8 skip to the Night Lord.
        if is_key_pressed(KeyCode::F2) {
            world.time = world.time.max(sim::content::MIDBOSS_AT - 3.0);
        }
        if is_key_pressed(KeyCode::F3) {
            world.debug_max_loadout();
        }
        if is_key_pressed(KeyCode::F4) && !world.paused() {
            world.open_chest(1);
        }
        if is_key_pressed(KeyCode::F5) && !world.paused() {
            world.open_chest(5);
        }
        if is_key_pressed(KeyCode::F6) {
            world.debug_all_weapons();
        }
        if is_key_pressed(KeyCode::F7) {
            world.shrine = Some(world.player.pos);
        }
        if is_key_pressed(KeyCode::F8) {
            world.time = world.time.max(sim::content::RUN_SECONDS - 3.0);
        }
        if let Some(s) = stress.as_mut() {
            let mut changed = true;
            if is_key_pressed(KeyCode::RightBracket) {
                s.enemies += 500;
            } else if is_key_pressed(KeyCode::LeftBracket) {
                s.enemies = s.enemies.saturating_sub(500);
            } else if is_key_pressed(KeyCode::Equal) {
                s.projectiles += 1000;
            } else if is_key_pressed(KeyCode::Minus) {
                s.projectiles = s.projectiles.saturating_sub(1000);
            } else {
                changed = false;
            }
            if changed {
                world.cfg.stress = Some(*s);
            }
        }

        // --- Fixed-timestep sim (paused while a menu or level-up is open) ---
        if screen == Screen::Playing && !world.paused() {
            acc += dt;
            let mut input = read_input();
            special_queued |= is_key_pressed(KeyCode::Space);
            input.special = special_queued;
            let t0 = get_time();
            let mut steps = 0;
            while acc >= DT && steps < 4 && !world.paused() {
                world.step(input);
                fx.absorb(&world.events);
                // Delivered exactly once, even if the sim steps several times this frame.
                input.special = false;
                special_queued = false;
                acc -= DT;
                steps += 1;
            }
            if !fx.low_hp_warned && world.player.hp < world.player.max_hp * 0.25 {
                fx.low_hp_warned = true;
                fx.say("Not... finished... yet.");
            }
            if steps == 4 {
                acc = 0.0; // too slow: drop backlog rather than spiral
            }
            if steps > 0 {
                sim_ms = sim_ms * 0.95 + ((get_time() - t0) * 1000.0 / steps as f64) as f32 * 0.05;
            }
            fx.update(dt);
        } else {
            acc = 0.0;
            // Space on a menu (e.g. dismissing a chest) must not fire Purify afterwards.
            special_queued = false;
        }

        // --- Render ---
        clear_background(Color::from_rgba(28, 24, 40, 255));
        let cam = render::camera_for(&world, &fx);
        render::draw_world(&world, &assets, &stage_art, &fx, &cam, now);
        set_default_camera();
        stage_art.draw_vignette();
        ui::hud(&world, &assets);
        ui::debug_overlay(get_fps(), sim_ms, &world, stress.is_some());

        if let (Some((text, left)), false) = (&fx.banner, world.paused()) {
            ui::banner(text, *left);
        }
        if let Some(items) = world.chest_result.clone() {
            let anim = chest_anim.get_or_insert_with(|| chest_ui::ChestAnim::new(items));
            if anim.update_and_draw(&assets, dt) {
                world.close_chest();
                chest_anim = None;
            }
        } else if let Some(offer) = world.shrine_offer.clone() {
            if let Some(choice) = ui::shrine(&offer) {
                world.choose_shrine(choice);
            }
        } else if let Some(offer) = world.offer.clone() {
            if let Some(i) = ui::level_up(&world, &offer, &assets) {
                world.choose(i);
            }
        } else if over {
            ui::results(&world, world.state == RunState::Won);
        } else if screen == Screen::Paused {
            ui::paused();
        }

        next_frame().await
    }
}
