use macroquad::prelude::*;
use sim::world::DT;
use sim::meta::{Profile, RunReport};
use sim::{Input, RunState, Stress, World, WorldConfig};

mod audio;
mod chest_ui;
mod kit;
mod menu;
mod render;
mod save;
mod sprites;
mod stage;
mod synth;
mod ui;
mod voice;

use render::{Assets, Fx};

fn window_conf() -> Conf {
    Conf { window_title: "Kizuna Survivors".into(), window_width: 1280, window_height: 720, ..Default::default() }
}

#[derive(PartialEq)]
enum Screen {
    /// Click-to-start splash (browsers need a click before audio/focus).
    Title,
    Menu,
    Characters,
    Shop,
    Collection,
    Settings,
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

fn new_world(stress: Option<Stress>, profile: &Profile) -> World {
    let seed = (macroquad::miniquad::date::now() * 1000.0) as u64;
    World::new(WorldConfig { seed, stress, hero: profile.selected, meta: profile.bonus(), ..Default::default() })
}

#[macroquad::main(window_conf)]
async fn main() {
    let assets = Assets::load();
    let stage_art = stage::StageArt::load();
    let mut fx = Fx::new();
    let mut screen = Screen::Title;
    let mut stress: Option<Stress> = None;
    let mut profile = save::load();
    let mut world = new_world(stress, &profile);
    // Set once a finished run has been banked into the profile.
    let mut report: Option<RunReport> = None;
    let mut hero_cursor = sim::content::ALL_HEROES.iter().position(|h| *h == profile.selected).unwrap_or(0);
    let mut shop_cursor = 0usize;
    let mut acc = 0.0f32;
    let mut sim_ms = 0.0f32;
    let mut chest_anim: Option<chest_ui::ChestAnim> = None;
    // Space is latched until the sim next steps; at high refresh rates many frames take no step.
    let mut special_queued = false;

    let mut collection = menu::CollectionState::default();

    kit::init();
    ui::loading();
    next_frame().await;
    let t_boot = get_time();
    ui::loading();
    next_frame().await;
    macroquad::logging::info!("fonts ready in {:.2}s", get_time() - t_boot);
    let t_audio = get_time();
    let mut audio = audio::Audio::load(profile.music_volume, profile.sfx_volume).await;
    macroquad::logging::info!("audio ready in {:.2}s", get_time() - t_audio);
    // Which run-end jingle has played (so it plays once).
    let mut jingle_played = false;

    loop {
        let dt = get_frame_time().min(0.1);
        let now = get_time();
        audio.begin_frame();
        // UI click feedback on any out-of-run screen.
        if !matches!(screen, Screen::Playing | Screen::Paused | Screen::Title) && (is_mouse_button_pressed(MouseButton::Left) || is_key_pressed(KeyCode::Enter)) {
            audio.play(synth::Sfx::Click);
        }
        match screen {
            Screen::Title | Screen::Playing | Screen::Paused => {}
            _ => audio.set_music(Some(synth::Track::Menu)),
        }

        // --- Out-of-run screens ---
        match screen {
            Screen::Title => {
                ui::title(now);
                if is_mouse_button_pressed(MouseButton::Left) || is_key_pressed(KeyCode::Enter) {
                    screen = Screen::Menu;
                }
            }
            Screen::Menu => match menu::main_menu(&profile, &assets, now) {
                Some(menu::MenuAction::Start) => screen = Screen::Characters,
                Some(menu::MenuAction::Shop) => screen = Screen::Shop,
                Some(menu::MenuAction::Collection) => screen = Screen::Collection,
                Some(menu::MenuAction::Settings) => screen = Screen::Settings,
                None => {}
            },
            Screen::Collection => {
                if menu::collection(&profile, &assets, &mut collection, now) {
                    screen = Screen::Menu;
                }
            }
            Screen::Settings => match menu::settings(&mut profile, now) {
                Some(menu::SettingsAction::Changed) => {
                    audio.music_volume = profile.music_volume;
                    audio.sfx_volume = profile.sfx_volume;
                    audio.apply_music_volume();
                    save::store(&profile);
                }
                Some(menu::SettingsAction::Back) => screen = Screen::Menu,
                None => {}
            },
            Screen::Characters => match menu::char_select(&mut profile, &assets, &mut hero_cursor, now) {
                Some(menu::SelectAction::Begin) => {
                    save::store(&profile);
                    world = new_world(stress, &profile);
                    report = None;
                    fx.set_hero(profile.selected);
                    fx.clear();
                    jingle_played = false;
                    screen = Screen::Playing;
                }
                Some(menu::SelectAction::Back) => screen = Screen::Menu,
                None => {}
            },
            Screen::Shop => match menu::shop(&mut profile, &assets, &mut shop_cursor, now) {
                Some(menu::ShopAction::Changed) => {
                    audio.play(synth::Sfx::Buy);
                    save::store(&profile);
                }
                Some(menu::ShopAction::Back) => screen = Screen::Menu,
                None => {}
            },
            Screen::Playing | Screen::Paused => {}
        }
        if !matches!(screen, Screen::Playing | Screen::Paused) {
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
        // Giving up from the pause menu ends the run normally (Sen is still banked).
        if screen == Screen::Paused && is_key_pressed(KeyCode::Q) {
            world.state = RunState::Dead;
            screen = Screen::Playing;
        }
        let over = matches!(world.state, RunState::Dead | RunState::Won);
        if over && report.is_none() && world.cfg.stress.is_none() {
            report = Some(profile.record_run(&world));
            save::store(&profile);
        }
        // Music: battle theme, the boss theme while the Night Lord lives, a jingle at the end.
        if over {
            audio.set_music(None);
            if !jingle_played {
                jingle_played = true;
                audio.play(if world.state == RunState::Won { synth::Sfx::Victory } else { synth::Sfx::Defeat });
            }
        } else {
            let boss = world.boss().is_some_and(|b| b.2 == sim::content::EnemyKind::NightLord);
            audio.set_music(Some(if boss { synth::Track::Boss } else { synth::Track::Battle }));
        }
        let restart = (is_key_pressed(KeyCode::R) && screen == Screen::Paused) || toggle_stress;
        if restart {
            world = new_world(stress, &profile);
            report = None;
            jingle_played = false;
            fx.clear();
            screen = Screen::Playing;
        }
        // Dev shortcuts (remove before release): F2 skip to the mid-boss, F3 max loadout, F4 open a chest,
        // F5 open a legendary chest, F6 every library weapon (ignores hero pool/slots), F7 shrine underfoot,
        // F8 skip to the Night Lord, F9 fill the special meter, F10 Awaken every owned weapon.
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
        if is_key_pressed(KeyCode::F10) {
            world.debug_awaken_all();
        }
        if is_key_pressed(KeyCode::F9) {
            world.player.special = 1.0;
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
                audio.absorb(&world.events);
                // Delivered exactly once, even if the sim steps several times this frame.
                input.special = false;
                special_queued = false;
                acc -= DT;
                steps += 1;
            }
            if !fx.low_hp_warned && world.player.hp < world.player.max_hp * 0.25 {
                fx.low_hp_warned = true;
                fx.say(fx.lines().low_hp);
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
        if screen != Screen::Paused {
            ui::hud(&world, &assets);
        }
        ui::debug_overlay(get_fps(), sim_ms, &world, stress.is_some());

        if let (Some((text, left)), false) = (&fx.banner, world.paused()) {
            ui::banner(text, *left);
        }
        if let Some(items) = world.chest_result.clone() {
            let anim = chest_anim.get_or_insert_with(|| chest_ui::ChestAnim::new(items));
            let done = anim.update_and_draw(&assets, dt);
            for cue in anim.cues.drain(..) {
                audio.play(cue);
            }
            if done {
                audio.play(synth::Sfx::Click);
                world.close_chest();
                chest_anim = None;
            }
        } else if let Some(offer) = world.shrine_offer.clone() {
            if let Some(choice) = ui::shrine(&offer, &assets) {
                audio.play(if choice.is_some() { synth::Sfx::Chime } else { synth::Sfx::Click });
                world.choose_shrine(choice);
            }
        } else if let Some(offer) = world.offer.clone() {
            if let Some(i) = ui::level_up(&world, &offer, &assets) {
                audio.play(synth::Sfx::Click);
                world.choose(i);
            }
        } else if over {
            match ui::results(&world, world.state == RunState::Won, report.as_ref(), &fx.lines(), &assets) {
                Some(ui::ResultsAction::Retry) => {
                    audio.play(synth::Sfx::Click);
                    world = new_world(stress, &profile);
                    report = None;
                    jingle_played = false;
                    fx.clear();
                    screen = Screen::Playing;
                }
                Some(ui::ResultsAction::Menu) => {
                    audio.play(synth::Sfx::Click);
                    screen = Screen::Menu;
                }
                None => {}
            }
        } else if screen == Screen::Paused {
            match ui::pause(&world, &assets, &mut profile.music_volume, &mut profile.sfx_volume) {
                Some(ui::PauseAction::Resume) => {
                    audio.play(synth::Sfx::Click);
                    screen = Screen::Playing;
                }
                Some(ui::PauseAction::Restart) => {
                    audio.play(synth::Sfx::Click);
                    world = new_world(stress, &profile);
                    report = None;
                    jingle_played = false;
                    fx.clear();
                    screen = Screen::Playing;
                }
                Some(ui::PauseAction::GiveUp) => {
                    audio.play(synth::Sfx::Click);
                    world.state = RunState::Dead;
                    screen = Screen::Playing;
                }
                Some(ui::PauseAction::Volume) => {
                    audio.music_volume = profile.music_volume;
                    audio.sfx_volume = profile.sfx_volume;
                    audio.apply_music_volume();
                    save::store(&profile);
                }
                None => {}
            }
        }

        next_frame().await
    }
}
