use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameRng {
    #[serde(skip, default = "default_rng")]
    inner: StdRng,
    seed: u64,
}

fn default_rng() -> StdRng {
    StdRng::seed_from_u64(0)
}

impl GameRng {
    pub fn new(seed: u64) -> Self {
        Self {
            inner: StdRng::seed_from_u64(seed),
            seed,
        }
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        slice.shuffle(&mut self.inner);
    }

    pub fn range(&mut self, min: i32, max: i32) -> i32 {
        self.inner.random_range(min..=max)
    }

    /// Generate a random u64 (useful for deriving sub-seeds).
    pub fn next_u64(&mut self) -> u64 {
        self.inner.random()
    }
}
