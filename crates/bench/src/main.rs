//! Headless sim benchmark: ms per 60 Hz tick at increasing densities.
use sim::{Input, Stress, Vec2, World, WorldConfig};
use std::time::Instant;

fn main() {
    println!("{:>8} {:>8} {:>10} {:>10} {:>10} {:>8}", "enemies", "proj", "avg ms", "p99 ms", "max ms", "kills");
    for &(enemies, proj) in &[(500, 1000), (2000, 4000), (3000, 6000), (5000, 8000)] {
        let mut w = World::new(WorldConfig { seed: 3, stress: Some(Stress { enemies, projectiles: proj }), ..Default::default() });
        let input = |t: u32| {
            let a = t as f32 * 0.004;
            Input { move_dir: Vec2::new(a.cos(), a.sin()), ..Default::default() }
        };
        for t in 0..300 {
            w.step(input(t));
        }
        let mut times = Vec::with_capacity(1800);
        for t in 300..2100 {
            let s = Instant::now();
            w.step(input(t));
            times.push(s.elapsed().as_secs_f64() * 1000.0);
        }
        times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let avg = times.iter().sum::<f64>() / times.len() as f64;
        println!(
            "{:>8} {:>8} {:>10.3} {:>10.3} {:>10.3} {:>8}",
            w.enemies.len(),
            w.projectiles.len(),
            avg,
            times[times.len() * 99 / 100],
            times[times.len() - 1],
            w.kills
        );
    }
}
