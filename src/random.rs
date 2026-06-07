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
///
/// `seed` is the generator's full internal state — public, since it's a bare
/// `Copy` value with no invariant to protect (any `u16` is a valid seed).
/// Read it to snapshot/restore a generator's position, or write it directly
/// to reseed; [`add_entropy`](Rng16::add_entropy) remains the convenient way
/// to perturb it without overwriting it outright.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rng16 {
    pub seed: u16,
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

    /// Adds `entropy` into the seed (wrapping) — a cheap way to fold in
    /// real-world randomness (e.g. sensor noise, timing jitter) without
    /// disturbing the generator's structure.
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

    /// Returns a random 16-bit value in `0..=65535`.
    #[inline]
    pub fn random16(&mut self) -> u16 {
        self.next_state()
    }

    /// Returns a random value within `range` — inclusive lower bound,
    /// exclusive upper bound, the same convention as
    /// [`rand::Rng::gen_range`](https://docs.rs/rand/latest/rand/trait.Rng.html#method.gen_range)
    /// (and as `core::ops::Range` itself).
    ///
    /// Works for both `u8` and `u16` (the only widths [`Rng16`] natively
    /// generates) — pass `0..limit` for "below `limit`" or `min..limit` for
    /// a shifted range; both collapse to a single call instead of FastLED's
    /// four separate `random{8,16}_{below,range}` functions.
    ///
    /// An empty or descending range (`range.end <= range.start`) wraps per
    /// the output type's unsigned arithmetic, matching the original C
    /// semantics — pass a non-empty ascending range to stay in
    /// well-defined territory.
    #[inline]
    pub fn gen_range<T: RangeSample>(&mut self, range: core::ops::Range<T>) -> T {
        T::sample_range(self, range)
    }
}

/// Integer widths [`Rng16::gen_range`] can produce — implemented for [`u8`]
/// and [`u16`], the two widths [`Rng16`]'s underlying generator natively
/// supports.
///
/// This is what lets `gen_range` be a single generic entry point rather than
/// a family of identically-shaped `random8_below`/`random16_range`/...
/// methods differing only in width; the trait dispatches to the right
/// scaling arithmetic for `Self`.
pub trait RangeSample: Sized + Copy {
    #[doc(hidden)]
    fn sample_range(rng: &mut Rng16, range: core::ops::Range<Self>) -> Self;
}

impl RangeSample for u8 {
    #[inline]
    fn sample_range(rng: &mut Rng16, range: core::ops::Range<u8>) -> u8 {
        let span = range.end.wrapping_sub(range.start);
        let r = rng.random8();
        let scaled = ((r as u16 * span as u16) >> 8) as u8;
        scaled.wrapping_add(range.start)
    }
}

impl RangeSample for u16 {
    #[inline]
    fn sample_range(rng: &mut Rng16, range: core::ops::Range<u16>) -> u16 {
        let span = range.end.wrapping_sub(range.start);
        let r = rng.random16();
        let scaled = ((span as u32 * r as u32) >> 16) as u16;
        scaled.wrapping_add(range.start)
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
            let a: u8 = rng.gen_range(0..10);
            let b: u8 = rng.gen_range(5..15);
            let c: u16 = rng.gen_range(0..1000);
            let d: u16 = rng.gen_range(100..200);
            assert!(a < 10);
            assert!((5..15).contains(&b));
            assert!(c < 1000);
            assert!((100..200).contains(&d));
        }
        assert_eq!(rng.gen_range(0u8..0), 0);
        assert_eq!(rng.gen_range(0u16..0), 0);
    }

    #[test]
    fn entropy_perturbs_state() {
        let mut a = Rng16::new(7);
        let mut b = a;
        b.add_entropy(0x1234);
        assert_ne!(a.seed, b.seed);
        assert_ne!(a.random16(), b.random16());
    }

    #[test]
    fn seed_field_can_be_read_and_overwritten_directly() {
        let mut rng = Rng16::new(0);
        rng.seed = 0xBEEF;
        assert_eq!(rng.seed, 0xBEEF);
    }
}
