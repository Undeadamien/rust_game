//! Tiny deterministic RNG and value noise.
//!
//! The whole world is generated from a single `u64` seed, so saves only need
//! to store the seed (plus the generated world for forward compatibility).

use serde::{Deserialize, Serialize};

/// SplitMix64. Fast, tiny, and good enough for a terminal game.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self {
            state: seed.wrapping_add(0x9E37_79B9_7F4A_7C15),
        }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform in `[0, 1)`.
    pub fn f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }

    /// Uniform integer in `[lo, hi)`.
    pub fn range(&mut self, lo: u32, hi: u32) -> u32 {
        debug_assert!(lo < hi);
        lo + (self.next_u64() % (hi - lo) as u64) as u32
    }

    pub fn chance(&mut self, p: f32) -> bool {
        self.f32() < p
    }

    pub fn choose<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        debug_assert!(!items.is_empty());
        &items[(self.next_u64() % items.len() as u64) as usize]
    }

    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            let j = (self.next_u64() % (i as u64 + 1)) as usize;
            items.swap(i, j);
        }
    }
}

fn mix(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn hash2(x: i32, y: i32, seed: u64) -> f32 {
    let h = mix(
        seed ^ (x as i64 as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
            ^ (y as i64 as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F),
    );
    (h >> 40) as f32 / (1u64 << 24) as f32
}

fn smooth(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

fn value_noise(x: f32, y: f32, seed: u64) -> f32 {
    let x0 = x.floor() as i32;
    let y0 = y.floor() as i32;
    let tx = smooth(x - x0 as f32);
    let ty = smooth(y - y0 as f32);

    let a = hash2(x0, y0, seed);
    let b = hash2(x0 + 1, y0, seed);
    let c = hash2(x0, y0 + 1, seed);
    let d = hash2(x0 + 1, y0 + 1, seed);

    let top = a + (b - a) * tx;
    let bottom = c + (d - c) * tx;
    top + (bottom - top) * ty
}

/// Fractal value noise in `[0, 1]`.
pub fn fbm(x: f32, y: f32, seed: u64, octaves: u32) -> f32 {
    let mut total = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut max = 0.0;

    for o in 0..octaves {
        total += value_noise(x * frequency, y * frequency, seed ^ (o as u64 * 0x9E37)) * amplitude;
        max += amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }

    total / max
}

/// Stable per-tile noise, used for small cosmetic variations.
pub fn tile_noise(x: i32, y: i32, seed: u64) -> f32 {
    hash2(x, y, seed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_numbers() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn ranges_are_in_bounds() {
        let mut rng = Rng::new(7);
        for _ in 0..1000 {
            let n = rng.range(3, 9);
            assert!((3..9).contains(&n));
            let f = rng.f32();
            assert!((0.0..1.0).contains(&f));
        }
    }
}
