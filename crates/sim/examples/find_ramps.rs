//! Dev: prints a ramp and a wall cell near the start of each terrain world, to stand the
//! hero there with `KZ_PERF_AT=x,y` (see docs/lighting.md).
use sim::campaign::StageId;
use sim::terrain::*;
fn main() {
    for st in [StageId::ShrineValley, StageId::EmberLeaf, StageId::PaleCitadel, StageId::Rainward, StageId::Shatterline] {
        let mut found = 0;
        'o: for r in 4i32..60 {
            for cy in -r..=r { for cx in -r..=r {
                if cx.abs() != r && cy.abs() != r { continue; }
                let c = cell_raw(st, cx, cy);
                if c & RAMP != 0 { println!("{:?} ramp {} {}", st, cx as f32 * CELL, cy as f32 * CELL); found += 1; if found >= 1 { break 'o; } }
            }}
        }
        // a cliff (wall, non-road)
        'p: for r in 4i32..40 { for cy in -r..=r { for cx in -r..=r {
            let c = cell_raw(st, cx, cy);
            if c & WALL != 0 && c & ROAD == 0 { println!("{:?} wall {} {}", st, cx as f32 * CELL, cy as f32 * CELL); break 'p; }
        }}}
    }
}
