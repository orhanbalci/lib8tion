//! Thin safe wrappers around a C transcription of FastLED's portable
//! `lib8tion` reference algorithms, compiled from `src/shim.c` via `cc` in
//! `build.rs`.
//!
//! This crate exists solely so the `lib8tion` crate's test suite can
//! differentially compare its `#![forbid(unsafe_code)]` Rust port against
//! the actual FastLED C behavior, without requiring `unsafe` in the crate
//! under test.

#![allow(non_snake_case)]

mod ffi {
    extern "C" {
        // math8
        pub fn fl_qadd8(i: u8, j: u8) -> u8;
        pub fn fl_qadd7(i: i8, j: i8) -> i8;
        pub fn fl_qsub8(i: u8, j: u8) -> u8;
        pub fn fl_add8(i: u8, j: u8) -> u8;
        pub fn fl_add8to16(i: u8, j: u16) -> u16;
        pub fn fl_sub8(i: u8, j: u8) -> u8;
        pub fn fl_avg8(i: u8, j: u8) -> u8;
        pub fn fl_avg16(i: u16, j: u16) -> u16;
        pub fn fl_avg8r(i: u8, j: u8) -> u8;
        pub fn fl_avg16r(i: u16, j: u16) -> u16;
        pub fn fl_avg7(i: i8, j: i8) -> i8;
        pub fn fl_avg15(i: i16, j: i16) -> i16;
        pub fn fl_mul8(i: u8, j: u8) -> u8;
        pub fn fl_qmul8(i: u8, j: u8) -> u8;
        pub fn fl_abs8(i: i8) -> i8;
        pub fn fl_blend8_8bit(a: u8, b: u8, amount_of_b: u8) -> u8;
        pub fn fl_blend8_16bit(a: u8, b: u8, amount_of_b: u8) -> u8;
        pub fn fl_blend8(a: u8, b: u8, amount_of_b: u8) -> u8;
        pub fn fl_mod8(a: u8, m: u8) -> u8;
        pub fn fl_addmod8(a: u8, b: u8, m: u8) -> u8;
        pub fn fl_submod8(a: u8, b: u8, m: u8) -> u8;
        pub fn fl_sqrt16(x: u16) -> u8;
        pub fn fl_sqrt8(x: u8) -> u8;

        // scale8
        pub fn fl_scale8(i: u8, scale: u8) -> u8;
        pub fn fl_scale8_video(i: u8, scale: u8) -> u8;
        pub fn fl_scale16by8(i: u16, scale: u8) -> u16;
        pub fn fl_scale16(i: u16, scale: u16) -> u16;
        pub fn fl_scale32by8(i: u32, scale: u8) -> u32;
        pub fn fl_dim8_raw(x: u8) -> u8;
        pub fn fl_dim8_video(x: u8) -> u8;
        pub fn fl_dim8_lin(x: u8) -> u8;
        pub fn fl_brighten8_raw(x: u8) -> u8;
        pub fn fl_brighten8_video(x: u8) -> u8;
        pub fn fl_brighten8_lin(x: u8) -> u8;

        // trig8
        pub fn fl_sin16(theta: u16) -> i16;
        pub fn fl_cos16(theta: u16) -> i16;
        pub fn fl_sin8(theta: u8) -> u8;
        pub fn fl_cos8(theta: u8) -> u8;

        // lerp / map
        pub fn fl_lerp8by8(a: u8, b: u8, frac: u8) -> u8;
        pub fn fl_lerp16by16(a: u16, b: u16, frac: u16) -> u16;
        pub fn fl_lerp16by8(a: u16, b: u16, frac: u8) -> u16;
        pub fn fl_lerp15by8(a: i16, b: i16, frac: u8) -> i16;
        pub fn fl_lerp15by16(a: i16, b: i16, frac: u16) -> i16;
        pub fn fl_map8(input: u8, range_start: u8, range_end: u8) -> u8;

        // ease / waveforms
        pub fn fl_ease8InOutQuad(i: u8) -> u8;
        pub fn fl_ease16InOutQuad(i: u16) -> u16;
        pub fn fl_ease8InOutCubic(i: u8) -> u8;
        pub fn fl_ease16InOutCubic(i: u16) -> u16;
        pub fn fl_ease8InOutApprox(i: u8) -> u8;
        pub fn fl_triwave8(i: u8) -> u8;
        pub fn fl_quadwave8(i: u8) -> u8;
        pub fn fl_cubicwave8(i: u8) -> u8;
        pub fn fl_squarewave8(i: u8, pulsewidth: u8) -> u8;
    }
}

macro_rules! wrap {
    ($(#[$meta:meta])* $name:ident($($arg:ident: $ty:ty),*) -> $ret:ty => $ffi:path) => {
        $(#[$meta])*
        pub fn $name($($arg: $ty),*) -> $ret {
            unsafe { $ffi($($arg),*) }
        }
    };
}

// math8
wrap!(qadd8(i: u8, j: u8) -> u8 => ffi::fl_qadd8);
wrap!(qadd7(i: i8, j: i8) -> i8 => ffi::fl_qadd7);
wrap!(qsub8(i: u8, j: u8) -> u8 => ffi::fl_qsub8);
wrap!(add8(i: u8, j: u8) -> u8 => ffi::fl_add8);
wrap!(add8to16(i: u8, j: u16) -> u16 => ffi::fl_add8to16);
wrap!(sub8(i: u8, j: u8) -> u8 => ffi::fl_sub8);
wrap!(avg8(i: u8, j: u8) -> u8 => ffi::fl_avg8);
wrap!(avg16(i: u16, j: u16) -> u16 => ffi::fl_avg16);
wrap!(avg8r(i: u8, j: u8) -> u8 => ffi::fl_avg8r);
wrap!(avg16r(i: u16, j: u16) -> u16 => ffi::fl_avg16r);
wrap!(avg7(i: i8, j: i8) -> i8 => ffi::fl_avg7);
wrap!(avg15(i: i16, j: i16) -> i16 => ffi::fl_avg15);
wrap!(mul8(i: u8, j: u8) -> u8 => ffi::fl_mul8);
wrap!(qmul8(i: u8, j: u8) -> u8 => ffi::fl_qmul8);
wrap!(abs8(i: i8) -> i8 => ffi::fl_abs8);
wrap!(blend8_8bit(a: u8, b: u8, amount_of_b: u8) -> u8 => ffi::fl_blend8_8bit);
wrap!(blend8_16bit(a: u8, b: u8, amount_of_b: u8) -> u8 => ffi::fl_blend8_16bit);
wrap!(blend8(a: u8, b: u8, amount_of_b: u8) -> u8 => ffi::fl_blend8);
wrap!(mod8(a: u8, m: u8) -> u8 => ffi::fl_mod8);
wrap!(addmod8(a: u8, b: u8, m: u8) -> u8 => ffi::fl_addmod8);
wrap!(submod8(a: u8, b: u8, m: u8) -> u8 => ffi::fl_submod8);
wrap!(sqrt16(x: u16) -> u8 => ffi::fl_sqrt16);
wrap!(sqrt8(x: u8) -> u8 => ffi::fl_sqrt8);

// scale8
wrap!(scale8(i: u8, scale: u8) -> u8 => ffi::fl_scale8);
wrap!(scale8_video(i: u8, scale: u8) -> u8 => ffi::fl_scale8_video);
wrap!(scale16by8(i: u16, scale: u8) -> u16 => ffi::fl_scale16by8);
wrap!(scale16(i: u16, scale: u16) -> u16 => ffi::fl_scale16);
wrap!(scale32by8(i: u32, scale: u8) -> u32 => ffi::fl_scale32by8);
wrap!(dim8_raw(x: u8) -> u8 => ffi::fl_dim8_raw);
wrap!(dim8_video(x: u8) -> u8 => ffi::fl_dim8_video);
wrap!(dim8_lin(x: u8) -> u8 => ffi::fl_dim8_lin);
wrap!(brighten8_raw(x: u8) -> u8 => ffi::fl_brighten8_raw);
wrap!(brighten8_video(x: u8) -> u8 => ffi::fl_brighten8_video);
wrap!(brighten8_lin(x: u8) -> u8 => ffi::fl_brighten8_lin);

// trig8
wrap!(sin16(theta: u16) -> i16 => ffi::fl_sin16);
wrap!(cos16(theta: u16) -> i16 => ffi::fl_cos16);
wrap!(sin8(theta: u8) -> u8 => ffi::fl_sin8);
wrap!(cos8(theta: u8) -> u8 => ffi::fl_cos8);

// lerp / map
wrap!(lerp8by8(a: u8, b: u8, frac: u8) -> u8 => ffi::fl_lerp8by8);
wrap!(lerp16by16(a: u16, b: u16, frac: u16) -> u16 => ffi::fl_lerp16by16);
wrap!(lerp16by8(a: u16, b: u16, frac: u8) -> u16 => ffi::fl_lerp16by8);
wrap!(lerp15by8(a: i16, b: i16, frac: u8) -> i16 => ffi::fl_lerp15by8);
wrap!(lerp15by16(a: i16, b: i16, frac: u16) -> i16 => ffi::fl_lerp15by16);
wrap!(map8(input: u8, range_start: u8, range_end: u8) -> u8 => ffi::fl_map8);

// ease / waveforms
wrap!(ease8_in_out_quad(i: u8) -> u8 => ffi::fl_ease8InOutQuad);
wrap!(ease16_in_out_quad(i: u16) -> u16 => ffi::fl_ease16InOutQuad);
wrap!(ease8_in_out_cubic(i: u8) -> u8 => ffi::fl_ease8InOutCubic);
wrap!(ease16_in_out_cubic(i: u16) -> u16 => ffi::fl_ease16InOutCubic);
wrap!(ease8_in_out_approx(i: u8) -> u8 => ffi::fl_ease8InOutApprox);
wrap!(triwave8(i: u8) -> u8 => ffi::fl_triwave8);
wrap!(quadwave8(i: u8) -> u8 => ffi::fl_quadwave8);
wrap!(cubicwave8(i: u8) -> u8 => ffi::fl_cubicwave8);
wrap!(squarewave8(i: u8, pulsewidth: u8) -> u8 => ffi::fl_squarewave8);
