//! Uniform spatial grid rebuilt every tick with a counting sort: O(n) build,
//! cache-friendly queries. The grid window follows the player; entities outside
//! it are clamped into the border cells, which stays correct (just slower) for them.

use crate::Vec2;

pub struct Grid {
    pub cell: f32,
    pub dim: usize,
    origin: Vec2,
    /// starts[c]..starts[c+1] is the slice of `items` in cell c.
    starts: Vec<u32>,
    items: Vec<u32>,
    cell_of: Vec<u32>,
}

impl Grid {
    pub fn new(cell: f32, dim: usize) -> Self {
        Grid { cell, dim, origin: Vec2::ZERO, starts: vec![0; dim * dim + 1], items: Vec::new(), cell_of: Vec::new() }
    }

    #[inline]
    fn coord(&self, v: f32, o: f32) -> usize {
        (((v - o) / self.cell) as isize).clamp(0, self.dim as isize - 1) as usize
    }

    #[inline]
    pub fn cell_xy(&self, p: Vec2) -> (usize, usize) {
        (self.coord(p.x, self.origin.x), self.coord(p.y, self.origin.y))
    }

    pub fn rebuild(&mut self, center: Vec2, positions: &[Vec2]) {
        let half = self.cell * self.dim as f32 * 0.5;
        self.origin = Vec2::new(center.x - half, center.y - half);
        self.starts.iter_mut().for_each(|s| *s = 0);
        self.cell_of.clear();
        for &p in positions {
            let (x, y) = self.cell_xy(p);
            let c = (y * self.dim + x) as u32;
            self.cell_of.push(c);
            self.starts[c as usize + 1] += 1;
        }
        for i in 1..self.starts.len() {
            self.starts[i] += self.starts[i - 1];
        }
        self.items.resize(positions.len(), 0);
        let mut fill = self.starts.clone();
        for (i, &c) in self.cell_of.iter().enumerate() {
            let slot = &mut fill[c as usize];
            self.items[*slot as usize] = i as u32;
            *slot += 1;
        }
    }

    #[inline]
    pub fn cell_items(&self, x: usize, y: usize) -> &[u32] {
        let c = y * self.dim + x;
        &self.items[self.starts[c] as usize..self.starts[c + 1] as usize]
    }

    /// Calls `f` for every item in cells overlapping the square of `radius` around `p`.
    #[inline]
    pub fn for_each_near(&self, p: Vec2, radius: f32, mut f: impl FnMut(u32)) {
        let (x0, y0) = self.cell_xy(Vec2::new(p.x - radius, p.y - radius));
        let (x1, y1) = self.cell_xy(Vec2::new(p.x + radius, p.y + radius));
        for y in y0..=y1 {
            for x in x0..=x1 {
                for &i in self.cell_items(x, y) {
                    f(i);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::Rng;

    #[test]
    fn near_query_matches_brute_force() {
        let mut rng = Rng::new(7);
        let pts: Vec<Vec2> = (0..3000).map(|_| Vec2::new(rng.range(-900.0, 900.0), rng.range(-900.0, 900.0))).collect();
        let mut g = Grid::new(32.0, 64);
        g.rebuild(Vec2::ZERO, &pts);
        for q in 0..50 {
            let p = pts[q * 37];
            let r = 40.0;
            let mut got = Vec::new();
            g.for_each_near(p, r, |i| {
                if (pts[i as usize] - p).length() <= r {
                    got.push(i)
                }
            });
            got.sort();
            let want: Vec<u32> = (0..pts.len() as u32).filter(|&i| (pts[i as usize] - p).length() <= r).collect();
            assert_eq!(got, want);
        }
    }
}
