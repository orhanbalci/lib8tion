//! A small, fast, *not cryptographically secure* pseudo-random number
//! generator, suitable for visually-random LED animation — significantly
//! cheaper than a "real" PRNG.
//!
//! This is a port of FastLED's `random8`/`random16` family. The original C
//! API keeps the generator state in a single global (`rand16seed`); this
//! port instead exposes [`Rng16`], an explicit, ownable, `Copy` generator
//! struct — the idiomatic `no_std` shape (no global mutable state, no
//! atomics/critical-sections required, trivially usable from multiple
//! independent call sites or tasks).
//!
//! The underlying generator is the same linear congruential generator:
//! `seed' = seed * 2053 + 13849`, with the output formed by mixing the high
//! and low bytes of the new seed for better distribution.

/// Multiplier for the linear congruential generator.
const RAND16_MULTIPLIER: u16 = 2053;
/// Increment for the linear congruential generator.
const RAND16_INCREMENT: u16 = 13849;

/// The default seed FastLED initializes its global generator with.
pub const DEFAULT_SEED: u16 = 1337;

/// A fast 16-bit linear-congruential pseudo-random number generator.
///
/// Construct one with an explicit seed via [`Rng16::new`], or use
/// [`Rng16::default`] for FastLED's default seed. Being a plain `Copy`
/// struct, it can live wherever is convenient — a `static` behind your own
/// synchronization primitive, a field on your animation state, a local
/// variable seeded per-frame, etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rng16 {
    seed: u16,
}

impl Default for Rng16 {
    /// Creates a generator seeded with FastLED's [`DEFAULT_SEED`].
    #[inline]
    fn default() -> Self {
        Self::new(DEFAULT_SEED)
    }
}

impl Rng16 {
    /// Creates a new generator with the given seed.
    ///
    /// Avoid seeding with `0` — like any LCG with an additive increment this
    /// one recovers immediately, but an all-zero seed is a degenerate
    /// starting point shared by nothing else, which is rarely what you want
    /// for visual variety across devices/runs.
    #[inline]
    pub const fn new(seed: u16) -> Self {
        Self { seed }
    }

    /// Returns the current internal seed/state.
    #[inline]
    pub const fn seed(&self) -> u16 {
        self.seed
    }

    /// Replaces the internal seed/state.
    #[inline]
    pub fn set_seed(&mut self, seed: u16) {
        self.seed = seed;
    }

    /// XORs `entropy` into the seed — a cheap way to fold in real-world
    /// randomness (e.g. sensor noise, timing jitter) without disturbing the
    /// generator's structure.
    #[inline]
    pub fn add_entropy(&mut self, entropy: u16) {
        self.seed = self.seed.wrapping_add(entropy);
    }

    /// Advances the LCG one step and returns the new raw 16-bit state.
    #[inline]
    fn next_state(&mut self) -> u16 {
        self.seed = self
            .seed
            .wrapping_mul(RAND16_MULTIPLIER)
            .wrapping_add(RAND16_INCREMENT);
        self.seed
    }

    /// Returns a random byte in `0..=255`.
    ///
    /// Mixes the high and low bytes of the new LCG state for better
    /// distribution and less sequential correlation than taking either byte
    /// alone.
    #[inline]
    pub fn random8(&mut self) -> u8 {
        let s = self.next_state();
        (s as u8).wrapping_add((s >> 8) as u8)
    }

    /// Returns a random byte in `0..limit` (exclusive).
    ///
    /// If `limit == 0`, always returns `0`.
    #[inline]
    pub fn random8_below(&mut self, limit: u8) -> u8 {
        let r = self.random8();
        ((r as u16 * limit as u16) >> 8) as u8
    }

    /// Returns a random byte in `min..limit` (inclusive lower, exclusive
    /// upper bound). If `limit <= min`, the range wraps per `u8` arithmetic,
    /// matching the original C semantics — pass `min < limit` to stay in
    /// well-defined territory.
    #[inline]
    pub fn random8_range(&mut self, min: u8, limit: u8) -> u8 {
        let delta = limit.wrapping_sub(min);
        self.random8_below(delta).wrapping_add(min)
    }

    /// Returns a random 16-bit value in `0..=65535`.
    #[inline]
    pub fn random16(&mut self) -> u16 {
        self.next_state()
    }

    /// Returns a random 16-bit value in `0..limit` (exclusive).
    ///
    /// If `limit == 0`, always returns `0`.
    #[inline]
    pub fn random16_below(&mut self, limit: u16) -> u16 {
        let r = self.random16();
        let p = limit as u32 * r as u32;
        (p >> 16) as u16
    }

    /// Returns a random 16-bit value in `min..limit` (inclusive lower,
    /// exclusive upper bound). If `limit <= min`, the range wraps per `u16`
    /// arithmetic, matching the original C semantics.
    #[inline]
    pub fn random16_range(&mut self, min: u16, limit: u16) -> u16 {
        let delta = limit.wrapping_sub(min);
        self.random16_below(delta).wrapping_add(min)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_for_a_given_seed() {
        let mut a = Rng16::new(1);
        let mut b = Rng16::new(1);
        for _ in 0..64 {
            assert_eq!(a.random8(), b.random8());
            assert_eq!(a.random16(), b.random16());
        }
    }

    #[test]
    fn different_seeds_diverge() {
        let mut a = Rng16::new(1);
        let mut b = Rng16::new(2);
        let seq_a: [u16; 8] = core::array::from_fn(|_| a.random16());
        let seq_b: [u16; 8] = core::array::from_fn(|_| b.random16());
        assert_ne!(seq_a, seq_b);
    }

    #[test]
    fn bounded_generation_stays_in_range() {
        let mut rng = Rng16::new(42);
        for _ in 0..2048 {
            assert!(rng.random8_below(10) < 10);
            assert!(rng.random8_range(5, 15) < 15);
            assert!(rng.random16_below(1000) < 1000);
            assert!(rng.random16_range(100, 200) < 200);
        }
        assert_eq!(rng.random8_below(0), 0);
        assert_eq!(rng.random16_below(0), 0);
    }

    #[test]
    fn entropy_perturbs_state() {
        let mut a = Rng16::new(7);
        let mut b = a;
        b.add_entropy(0x1234);
        assert_ne!(a.seed(), b.seed());
        assert_ne!(a.random16(), b.random16());
    }

    #[test]
    fn seed_accessors_round_trip() {
        let mut rng = Rng16::new(0);
        rng.set_seed(0xBEEF);
        assert_eq!(rng.seed(), 0xBEEF);
    }
}
