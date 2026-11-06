use macroquad::prelude::*;
use sim::world::DT;
use sim::meta::{Profile, RunReport};
use sim::{Input, RunState, Stress, World, WorldConfig};

mod armory;
mod audio;
mod chest_ui;
mod kit;
mod menu;
mod platform;
mod render;
mod save;
mod sprites;
mod stage;
mod synth;
mod touch;
mod ui;
mod voice;

use render::{Assets, Fx};

fn window_conf() -> Conf {
    // high_dpi: render at the device's real pixel density (sharp text on phones and retina screens).
    Conf { window_title: "Kizuna Survivors".into(), window_width: 1280, window_height: 720, high_dpi: true, ..Default::default() }
}

#[derive(PartialEq)]
enum Screen {
    /// Click-to-start splash (browsers need a click before audio/focus).
    Title,
    Menu,
    /// Map select (world + difficulty), before character select.
    Stages,
    Characters,
    Armory,
    Draw,
    Shop,
    Collection,
    Stats,
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
    {
    let ascension = if profile.difficulty == sim::campaign::Difficulty::Nightmare { profile.ascension.min(profile.ascension_open(profile.stage)) } else { 0 };
    World::new(WorldConfig {
        seed,
        stress,
        hero: profile.selected,
        meta: profile.bonus(),
        stage: profile.stage,
        difficulty: profile.difficulty,
        ascension,
        gear: profile.loadout(profile.selected),
        ..Default::default()
    })
    }
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
    let mut armory_state = armory::ArmoryState::default();
    let mut draw_state = armory::DrawState::default();
    // The big map view is open (the run pauses meanwhile).
    let mut map_open = false;
    let mut map_opened_now = false;
    let mut touch = touch::TouchControls::default();

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
        // UI draws in virtual 1280x720 pixels (scaled down on small screens).
        kit::begin_ui();
        let in_run = screen == Screen::Playing && !matches!(world.state, RunState::Dead | RunState::Won) && !world.paused();
        touch.update(in_run && !map_open);
        if touch.pause && in_run {
            screen = Screen::Paused;
        }
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
        // Dev (remove before release): F7 on the menu or character select unlocks every hero.
        if matches!(screen, Screen::Menu | Screen::Characters | Screen::Stages) && is_key_pressed(KeyCode::F7) {
            for h in sim::content::ALL_HEROES {
                if !profile.unlocked.contains(&h) {
                    profile.unlocked.push(h);
                }
            }
            save::store(&profile);
            audio.play(synth::Sfx::Buy);
        }
        // Dev (remove before release): F6 on the map select opens every world and difficulty.
        if screen == Screen::Stages && is_key_pressed(KeyCode::F6) {
            for row in profile.cleared.iter_mut() {
                row[0] = true;
                row[1] = true;
            }
            save::store(&profile);
            audio.play(synth::Sfx::Buy);
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
                Some(menu::MenuAction::Start) => screen = Screen::Stages,
                Some(menu::MenuAction::Shop) => screen = Screen::Shop,
                Some(menu::MenuAction::Armory) => {
                    armory_state.hero = profile.selected.index();
                    screen = Screen::Armory;
                }
                Some(menu::MenuAction::Draw) => screen = Screen::Draw,
                Some(menu::MenuAction::Collection) => screen = Screen::Collection,
                Some(menu::MenuAction::Stats) => screen = Screen::Stats,
                Some(menu::MenuAction::Settings) => screen = Screen::Settings,
                None => {}
            },
            Screen::Collection => {
                if menu::collection(&profile, &assets, &mut collection, now) {
                    screen = Screen::Menu;
                }
            }
            Screen::Stats => {
                if menu::stats(&profile, &assets, now) {
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
            Screen::Armory => match armory::armory(&mut profile, &assets, &mut armory_state, now) {
                Some(armory::ArmoryAction::Changed) => save::store(&profile),
                Some(armory::ArmoryAction::Back) => {
                    save::store(&profile);
                    screen = Screen::Menu;
                }
                None => {}
            },
            Screen::Draw => match armory::shrine_draw(&mut profile, &assets, &mut draw_state, now) {
                Some(armory::DrawAction::Drew(best)) => {
                    audio.play(if best >= sim::gear::Rarity::Legendary { synth::Sfx::Awaken } else { synth::Sfx::Chime });
                    save::store(&profile);
                }
                Some(armory::DrawAction::Changed) => save::store(&profile),
                Some(armory::DrawAction::Back) => {
                    save::store(&profile);
                    screen = Screen::Menu;
                }
                None => {}
            },
            Screen::Stages => match menu::stage_select(&mut profile, &assets, &stage_art, now) {
                Some(menu::StageAction::Go) => {
                    save::store(&profile);
                    screen = Screen::Characters;
                }
                Some(menu::StageAction::Back) => {
                    save::store(&profile);
                    screen = Screen::Menu;
                }
                None => {}
            },
            Screen::Characters => match menu::char_select(&mut profile, &assets, &mut hero_cursor, now) {
                Some(menu::SelectAction::Begin) => {
                    save::store(&profile);
                    if profile.stage.index() > 0 {
                        audio.prepare(synth::Track::World(profile.stage.index() as u8)).await;
                    }
                    world = new_world(stress, &profile);
                    report = None;
                    fx.set_hero(profile.selected);
                    fx.accent = stage::accent(profile.stage);
                    fx.clear();
                    jingle_played = false;
                    screen = Screen::Playing;
                }
                Some(menu::SelectAction::Back) => screen = Screen::Stages,
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
        if is_key_pressed(KeyCode::Escape) && !over && !map_open && (screen == Screen::Paused || !world.paused()) {
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
            let boss = world.boss().is_some_and(|b| b.2.rank() >= sim::content::Rank::Boss);
            let theme = match world.cfg.stage.index() {
                0 => synth::Track::Battle,
                i => synth::Track::World(i as u8),
            };
            audio.set_music(Some(if boss { synth::Track::Boss } else { theme }));
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
        // F8 skip to the final boss, B skip to the second boss, F9 fill the special meter, F10 Awaken every owned weapon,
        // P drop a random pickup nearby, O drop a chest nearby.
        if is_key_pressed(KeyCode::F2) {
            world.time = world.time.max(world.stage().mini_at - 3.0);
        }
        // Dev: B skips to the stage's second boss.
        if is_key_pressed(KeyCode::B) {
            if let Some((_, at)) = world.stage().boss {
                world.time = world.time.max(at - 3.0);
            }
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
        if is_key_pressed(KeyCode::O) && screen == Screen::Playing {
            // Dev: drop a treasure chest nearby (sometimes a boss chest).
            let tier = if rand::gen_range(0, 3) == 0 { 3 } else { 1 };
            world.chests.push((world.player.pos + sim::Vec2::new(90.0, -40.0), tier));
        }
        if is_key_pressed(KeyCode::P) {
            // Dev: drop a random floor pickup near the player.
            let k = sim::content::PickupKind::roll(rand::gen_range(0.0, 1.0));
            let a = rand::gen_range(0.0, std::f32::consts::TAU);
            world.pickups.push((world.player.pos + sim::Vec2::from_angle(a) * 70.0, k));
        }
        if is_key_pressed(KeyCode::F10) {
            world.debug_awaken_all();
        }
        if is_key_pressed(KeyCode::F9) {
            world.player.special = 1.0;
        }
        if is_key_pressed(KeyCode::F8) {
            world.time = world.time.max(world.stage().final_at - 3.0);
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
        // Map toggle (M / Tab) during play.
        if screen == Screen::Playing && !over && !world.paused() && (is_key_pressed(KeyCode::M) || is_key_pressed(KeyCode::Tab)) {
            map_open = !map_open;
        }
        if screen != Screen::Playing || over {
            map_open = false;
        }
        if screen == Screen::Playing && !world.paused() && !map_open {
            acc += dt;
            let mut input = read_input();
            input.move_dir = input.move_dir + touch.dir();
            special_queued |= is_key_pressed(KeyCode::Space) || touch.special;
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
            fx.track_trail(&world, dt);
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
        if world.stage().mechanic == sim::campaign::Mechanic::Downpour {
            let pp = cam.world_to_screen(vec2(world.player.pos.x, world.player.pos.y));
            stage_art.draw_downpour(pp, now, fx.flash);
        }
        stage_art.draw_vignette();
        kit::begin_ui();
        if screen != Screen::Paused && !map_open {
            ui::hud(&world, &assets);
            map_opened_now = false;
            if !map_open && !over && !world.paused() && ui::minimap(&world, &assets, false, false) {
                map_open = true;
                map_opened_now = true;
            }
        }
        ui::debug_overlay(get_fps(), sim_ms, &world, stress.is_some());
        if in_run && !map_open && screen == Screen::Playing {
            let form = world.form_kind().map(|k| world.player.form / k.duration());
            touch.draw(world.player.special, world.player.special >= 1.0, form);
        }

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
        } else if map_open {
            if ui::minimap(&world, &assets, true, map_opened_now) {
                map_open = false;
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
