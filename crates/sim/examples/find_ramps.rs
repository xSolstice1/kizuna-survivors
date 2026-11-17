//! Dev: prints a few stairs (ramp cells) and walls near the start of each terrain world, to
//! stand the hero there with `KZ_PERF_AT=x,y` (see docs/graphics.md).
use sim::campaign::StageId;
use sim::terrain::*;
fn main() {
    for st in [StageId::ShrineValley, StageId::EmberLeaf, StageId::PaleCitadel, StageId::Rainward, StageId::Shatterline] {
        let mut ramps: Vec<(i32, i32)> = Vec::new();
        let mut walls: Vec<(i32, i32)> = Vec::new();
        for r in 4i32..70 {
            for cy in -r..=r {
                for cx in -r..=r {
                    if cx.abs() != r && cy.abs() != r {
                        continue;
                    }
                    let c = cell_raw(st, cx, cy);
                    let far = |v: &Vec<(i32, i32)>| v.iter().all(|&(x, y)| (x - cx).abs() + (y - cy).abs() > 30);
                    if c & RAMP != 0 && ramps.len() < 3 && far(&ramps) {
                        ramps.push((cx, cy));
                    }
                    if c & WALL != 0 && c & ROAD == 0 && walls.len() < 2 && far(&walls) {
                        walls.push((cx, cy));
                    }
                }
            }
        }
        for (cx, cy) in ramps {
            println!("{} ramp {},{}", st.key(), cx as f32 * CELL, cy as f32 * CELL);
        }
        for (cx, cy) in walls {
            println!("{} wall {},{}", st.key(), cx as f32 * CELL, cy as f32 * CELL);
        }
    }
}
