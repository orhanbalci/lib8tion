//! Generic integer range mapping: rescale a value from one integer width to
//! another while preserving its relative position in the range
//! (e.g. "50% of an 8-bit range" maps to "50% of a 16-bit range").
//!
//! Scaling *up* (8→16, 8→32, 16→32) is done via bit replication
//! (`0xAB → 0xABAB`), which maps both endpoints exactly. Scaling *down*
//! (16→8, 32→16, 32→8) is done via a rounded right-shift, which is nearly
//! identical to (but much cheaper than) floating-point division.
//!
//! This is a port of FastLED's `int_scale<FROM, TO>()`. Where the C++
//! version dispatches on explicit template specializations, this version
//! uses a sealed [`IntScale`] trait so the compiler still rejects
//! unsupported `(FROM, TO)` pairs at compile time.

mod sealed {
    pub trait Sealed {}
}

/// Implemented for every supported `(from, to)` integer pair for
/// [`int_scale`]. This trait is sealed — it cannot be implemented outside of
/// this crate — so unsupported conversions are rejected at compile time, just
/// like the C++ template specializations they replace.
///
/// Brought into scope at the crate root, so the postfix form
/// `x.int_scale()` works directly wherever the target type is clear from
/// context (a binding's type annotation, a function argument, ...) — the same
/// shape as [`Into::into`]. Reach for the free function [`int_scale`] when you
/// need to spell the target type out explicitly instead.
///
/// ```
/// use lib8tion::IntScale;
///
/// let x: u8 = 0xAB;
/// let y: u16 = x.int_scale();
/// assert_eq!(y, 0xABAB);
/// ```
pub trait IntScale<To>: sealed::Sealed {
    /// Rescale `self` from this type's range into `To`'s range, preserving
    /// relative position.
    fn int_scale(self) -> To;
}

/// Rescale `x` from the full range of `From` to the full range of `To`,
/// preserving its relative position (e.g. `int_scale::<u8, u16>(0x80)` is
/// `0x8080`, the 16-bit value at the same relative position as `0x80` in the
/// 8-bit range).
///
/// Both type parameters must be given explicitly to avoid masking bugs with
/// implicit conversions: `int_scale::<u8, u16>(x)`, not `int_scale(x)`. When
/// the target type is already clear from context, [`IntScale::int_scale`]'s
/// postfix form (`x.int_scale()`) reads more naturally — this free function
/// exists for the cases where spelling out both ends explicitly is clearer
/// (e.g. inside a generic helper, or simply for readability at the call site).
#[inline(always)]
pub fn int_scale<From, To>(x: From) -> To
where
    From: IntScale<To>,
{
    x.int_scale()
}

macro_rules! impl_sealed {
    ($($t:ty),* $(,)?) => {
        $(impl sealed::Sealed for $t {})*
    };
}
impl_sealed!(u8, i8, u16, i16, u32, i32);

/// Implements upscaling via bit replication: multiply by the repunit that
/// tiles the low bit-pattern across the wider type
/// (`0x101` for 8→16, `0x1010101` for 8→32, `0x10001` for 16→32).
macro_rules! impl_scale_up {
    ($from:ty, $to:ty, $unsigned_from:ty, $unsigned_to:ty, $repunit:expr) => {
        impl IntScale<$to> for $from {
            #[inline(always)]
            fn int_scale(self) -> $to {
                let bits = self as $unsigned_from;
                ((bits as $unsigned_to) * $repunit) as $to
            }
        }
    };
}

impl_scale_up!(u8, u16, u8, u16, 0x101);
impl_scale_up!(i8, i16, u8, u16, 0x101);
impl_scale_up!(u8, i16, u8, u16, 0x101);
impl_scale_up!(i8, u16, u8, u16, 0x101);

impl_scale_up!(u8, u32, u8, u32, 0x0101_0101);
impl_scale_up!(i8, i32, u8, u32, 0x0101_0101);
impl_scale_up!(u8, i32, u8, u32, 0x0101_0101);
impl_scale_up!(i8, u32, u8, u32, 0x0101_0101);

impl_scale_up!(u16, u32, u16, u32, 0x0001_0001);
impl_scale_up!(i16, i32, u16, u32, 0x0001_0001);
impl_scale_up!(u16, i32, u16, u32, 0x0001_0001);
impl_scale_up!(i16, u32, u16, u32, 0x0001_0001);

/// Implements downscaling for an *unsigned* source type via a rounded right
/// shift (add half the dropped range before shifting, so values round to
/// nearest rather than truncating), saturating at `$threshold` so the input
/// maximum maps exactly to the output maximum. The intermediate arithmetic is
/// carried out in `u64` — wide enough that it never wraps for any
/// non-saturating input — exactly mirroring the promotion to `int`/`unsigned`
/// that the original C performs before narrowing.
macro_rules! impl_scale_down_from_unsigned {
    ($from:ty, $to:ty, $shift:expr, $round:expr, $threshold:expr, $sat:expr) => {
        impl IntScale<$to> for $from {
            #[inline(always)]
            fn int_scale(self) -> $to {
                if self >= $threshold {
                    return $sat as $to;
                }
                ((self as u64 + $round as u64) >> $shift) as $to
            }
        }
    };
}

/// Implements downscaling for a *signed* source type. The saturation check
/// and the rounding/shift must both be performed on the signed value (not its
/// bit-reinterpretation) — otherwise negative inputs would incorrectly
/// compare as "large" and saturate. `i64` is wide enough to hold `self +
/// round` without overflow for any non-saturating input, and the final `as
/// $to` truncates and reinterprets exactly like C's narrowing cast.
macro_rules! impl_scale_down_from_signed {
    ($from:ty, $to:ty, $shift:expr, $round:expr, $threshold:expr, $sat:expr) => {
        impl IntScale<$to> for $from {
            #[inline(always)]
            fn int_scale(self) -> $to {
                if self >= $threshold {
                    return $sat as $to;
                }
                ((self as i64 + $round as i64) >> $shift) as $to
            }
        }
    };
}

// u16 -> u8 / i8
impl_scale_down_from_unsigned!(u16, u8, 8, 128u32, 0xff00u16, 0xffu8);
impl_scale_down_from_unsigned!(u16, i8, 8, 128u32, 0xff00u16, 0xffu8);
// i16 -> i8 / u8
impl_scale_down_from_signed!(i16, i8, 8, 128i32, 0x7f80i16, 127i8);
impl_scale_down_from_signed!(i16, u8, 8, 128i32, 0x7f80i16, 0xffu8);

// u32 -> u16 / i16
impl_scale_down_from_unsigned!(u32, u16, 16, 32768u32, 0xffff_0000u32, 0xffffu16);
impl_scale_down_from_unsigned!(u32, i16, 16, 32768u32, 0xffff_0000u32, 0xffffu16);
// i32 -> i16 / u16
impl_scale_down_from_signed!(i32, i16, 16, 32768i32, 0x7fff_8000i32, 32767i16);
impl_scale_down_from_signed!(i32, u16, 16, 32768i32, 0x7fff_8000i32, 0xffffu16);

// u32 -> u8 / i8
impl_scale_down_from_unsigned!(u32, u8, 24, 0x0080_0000u32, 0xff00_0000u32, 0xffu8);
impl_scale_down_from_unsigned!(u32, i8, 24, 0x0080_0000u32, 0xff00_0000u32, 0x7fi8);
// i32 -> i8 / u8
impl_scale_down_from_signed!(i32, i8, 24, 0x0080_0000i32, 0x7f00_0000i32, 127i8);
impl_scale_down_from_signed!(i32, u8, 24, 0x0080_0000i32, 0x7f00_0000i32, 0xffu8);

/// Identity scaling: `int_scale::<T, T>(x) == x`.
macro_rules! impl_scale_identity {
    ($($t:ty),* $(,)?) => {
        $(impl IntScale<$t> for $t {
            #[inline(always)]
            fn int_scale(self) -> $t { self }
        })*
    };
}
impl_scale_identity!(u8, i8, u16, i16, u32, i32);

/// Maps an 8-bit unsigned value to a 16-bit unsigned value.
#[deprecated(note = "use int_scale::<u8, u16>(x) instead")]
#[inline(always)]
pub fn map8_to_16(x: u8) -> u16 {
    int_scale::<u8, u16>(x)
}

/// Maps an 8-bit unsigned value to a 32-bit unsigned value.
#[deprecated(note = "use int_scale::<u8, u32>(x) instead")]
#[inline(always)]
pub fn map8_to_32(x: u8) -> u32 {
    int_scale::<u8, u32>(x)
}

/// Maps a 16-bit unsigned value to a 32-bit unsigned value.
#[deprecated(note = "use int_scale::<u16, u32>(x) instead")]
#[inline(always)]
pub fn map16_to_32(x: u16) -> u32 {
    int_scale::<u16, u32>(x)
}

/// Maps a 16-bit unsigned value down to an 8-bit unsigned value.
#[deprecated(note = "use int_scale::<u16, u8>(x) instead")]
#[inline(always)]
pub fn map16_to_8(x: u16) -> u8 {
    int_scale::<u16, u8>(x)
}

/// Maps a 32-bit unsigned value down to a 16-bit unsigned value.
#[deprecated(note = "use int_scale::<u32, u16>(x) instead")]
#[inline(always)]
pub fn map32_to_16(x: u32) -> u16 {
    int_scale::<u32, u16>(x)
}

/// Maps a 32-bit unsigned value down to an 8-bit unsigned value.
#[deprecated(note = "use int_scale::<u32, u8>(x) instead")]
#[inline(always)]
pub fn map32_to_8(x: u32) -> u8 {
    int_scale::<u32, u8>(x)
}

/// Maps an 8-bit signed value to a 16-bit signed value.
#[deprecated(note = "use int_scale::<i8, i16>(x) instead")]
#[inline(always)]
pub fn smap8_to_16(x: i8) -> i16 {
    int_scale::<i8, i16>(x)
}

/// Maps an 8-bit signed value to a 32-bit signed value.
#[deprecated(note = "use int_scale::<i8, i32>(x) instead")]
#[inline(always)]
pub fn smap8_to_32(x: i8) -> i32 {
    int_scale::<i8, i32>(x)
}

/// Maps a 16-bit signed value to a 32-bit signed value.
#[deprecated(note = "use int_scale::<i16, i32>(x) instead")]
#[inline(always)]
pub fn smap16_to_32(x: i16) -> i32 {
    int_scale::<i16, i32>(x)
}

/// Maps a 16-bit signed value down to an 8-bit signed value.
#[deprecated(note = "use int_scale::<i16, i8>(x) instead")]
#[inline(always)]
pub fn smap16_to_8(x: i16) -> i8 {
    int_scale::<i16, i8>(x)
}

/// Maps a 32-bit signed value down to a 16-bit signed value.
#[deprecated(note = "use int_scale::<i32, i16>(x) instead")]
#[inline(always)]
pub fn smap32_to_16(x: i32) -> i16 {
    int_scale::<i32, i16>(x)
}

/// Maps a 32-bit signed value down to an 8-bit signed value.
#[deprecated(note = "use int_scale::<i32, i8>(x) instead")]
#[inline(always)]
pub fn smap32_to_8(x: i32) -> i8 {
    int_scale::<i32, i8>(x)
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::*;

    #[test]
    fn scale_up_replicates_bits() {
        assert_eq!(int_scale::<u8, u16>(0x00), 0x0000);
        assert_eq!(int_scale::<u8, u16>(0xFF), 0xFFFF);
        assert_eq!(int_scale::<u8, u16>(0xAB), 0xABAB);
        assert_eq!(int_scale::<u8, u32>(0xAB), 0xABAB_ABAB);
        assert_eq!(int_scale::<u16, u32>(0xABCD), 0xABCD_ABCD);
    }

    #[test]
    fn scale_down_rounds_and_saturates() {
        assert_eq!(int_scale::<u16, u8>(0x0000), 0x00);
        assert_eq!(int_scale::<u16, u8>(0xFFFF), 0xFF);
        assert_eq!(int_scale::<u16, u8>(0x7F80), 0x80); // rounds up at midpoint
        assert_eq!(int_scale::<u16, u8>(0x7F7F), 0x7F);
        assert_eq!(int_scale::<u32, u16>(0xFFFF_FFFF), 0xFFFF);
        assert_eq!(int_scale::<u32, u8>(0xFFFF_FFFF), 0xFF);
    }

    #[test]
    fn scale_down_signed_handles_negative_values_correctly() {
        // Negative inputs must NOT be treated as "large" by the saturation
        // check (a naive bit-reinterpretation bug would saturate these).
        assert_eq!(int_scale::<i16, i8>(-1), 0);
        assert_eq!(int_scale::<i16, i8>(-256), -1);
        assert_eq!(int_scale::<i16, i8>(i16::MIN), i8::MIN);
        assert_eq!(int_scale::<i16, i8>(i16::MAX), 127);
        assert_eq!(int_scale::<i16, i8>(0), 0);

        assert_eq!(int_scale::<i32, i16>(-1), 0);
        assert_eq!(int_scale::<i32, i16>(i32::MIN), i16::MIN);
        assert_eq!(int_scale::<i32, i16>(i32::MAX), 32767);

        assert_eq!(int_scale::<i32, i8>(-1), 0);
        assert_eq!(int_scale::<i32, i8>(i32::MIN), i8::MIN);
        assert_eq!(int_scale::<i32, i8>(i32::MAX), 127);
    }

    #[test]
    fn identity_is_noop() {
        assert_eq!(int_scale::<u8, u8>(123), 123);
        assert_eq!(int_scale::<i32, i32>(-7), -7);
    }

    #[test]
    fn postfix_form_matches_the_free_function() {
        let x: u8 = 0xAB;
        let up: u16 = x.int_scale();
        assert_eq!(up, int_scale::<u8, u16>(x));

        let down: u8 = 0x1234u16.int_scale();
        assert_eq!(down, int_scale::<u16, u8>(0x1234));
    }

    #[test]
    fn legacy_named_wrappers_match_int_scale() {
        assert_eq!(map8_to_16(0xAB), int_scale::<u8, u16>(0xAB));
        assert_eq!(map16_to_8(0x1234), int_scale::<u16, u8>(0x1234));
        assert_eq!(smap8_to_16(-1), int_scale::<i8, i16>(-1));
        assert_eq!(smap32_to_8(-1), int_scale::<i32, i8>(-1));
    }
}
