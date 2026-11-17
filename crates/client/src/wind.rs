//! One wind for the whole world: grass and drifting motes all bend to the same
//! gusts, so a gust is seen rolling across the scene instead of each thing swaying alone.

use macroquad::prelude::*;

/// Which way the wind blows (world units, unit length): from the west, a little south.
pub const DIR: Vec2 = Vec2::new(0.92, -0.38);

/// The gust at `p` at time `t` (0 calm .. 1 full): slow fronts rolling downwind.
pub fn gust(p: Vec2, t: f32) -> f32 {
    let along = p.dot(DIR);
    let g = (t * 0.9 - along * 0.012).sin() * 0.5 + 0.5;
    let slow = (t * 0.23 - along * 0.003).sin() * 0.5 + 0.5;
    g * g * (0.55 + 0.45 * slow)
}
