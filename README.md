# lib8tion

Fast 8-/16-bit fixed-point math for `no_std` embedded targets — a Rust port
of the math primitives from [FastLED's `lib8tion`](https://github.com/FastLED/FastLED/tree/master/src/lib8tion).

```toml
[dependencies]
lib8tion = "0.1"
```

`#![no_std]`, `#![forbid(unsafe_code)]`, no allocation, no panics. Every
function is pure integer math with overflow behavior chosen to match the
original C implementation (saturating, wrapping, or truncating — whichever
FastLED's algorithm relies on), so animations ported from FastLED/Arduino
sketches produce identical output.

## Why

LED animation code leans on cheap, branch-light integer math: scale a byte by
a fraction, blend two colors, approximate a sine wave, oscillate a value at a
given BPM. FastLED's `lib8tion` is the reference implementation of these
tricks for AVR/ARM microcontrollers; this crate brings the same primitives —
and the same bit-for-bit behavior — to `no_std` Rust.

## What's here

| Module | Contents |
| --- | --- |
| [`math8`](src/math8.rs) | Saturating/wrapping byte arithmetic: `qadd8`, `qsub8`, `add8`, `avg8`, `mul8`, `qmul8`, `abs8`, `blend8`, `mod8`, `sqrt8`/`sqrt16`, ... |
| [`scale8`](src/scale8.rs) | Fast scaling & dimming: `scale8`, `scale8_video`, `scale16`, `nscale8x3`/`nscale8` (in place, incl. whole-slice), `dim8_*` / `brighten8_*` |
| [`trig8`](src/trig8.rs) | Lookup-table approximations of `sin`/`cos`: `sin8`, `cos8`, `sin16`, `cos16` |
| [`lerp`](src/lerp.rs) | Linear interpolation & range mapping: `lerp8by8`, `lerp16by8`, `lerp16by16`, `lerp15by8`, `lerp15by16`, `map8` |
| [`ease`](src/ease.rs) | Easing curves & waveform generators: `ease8_in_out_quad`/`cubic`/`approx`, `ease16_in_out_quad`/`cubic`, `triwave8`, `quadwave8`, `cubicwave8`, `squarewave8` |
| [`beat`](src/beat.rs) | BPM-driven phase/value oscillators: `beat8`, `beat16`, `beat88`, `beatsin8`, `beatsin16`, `beatsin88` |
| [`intmap`](src/intmap.rs) | Generic integer range remapping between bit widths: `int_scale` |
| [`fixed_point`](src/fixed_point.rs) | `Qfx<F>` fixed-point scale-factor type (`Q44`, `Q62`, `Q88`, `Q124`) |
| [`random`](src/random.rs) | `Rng16` — a small, fast, explicitly-seeded PRNG (not cryptographically secure) |

All of the above are re-exported from the crate root, so `lib8tion::scale8`,
`lib8tion::sin8`, `lib8tion::beat8`, etc. all work directly.

## Design notes

- **No hidden global state.** FastLED's C functions read implicit global
  state — `millis()` for the beat generators, a static seed for `random8()`.
  This port instead takes that state as an explicit argument
  (`now_millis: u32` for [`beat8`](src/beat.rs) & friends, an explicit seed
  for [`Rng16`](src/random.rs)) — the only sensible shape in `no_std`, where
  there's no universal monotonic clock or global allocator to reach for.
  Drive the beat generators with whatever millisecond counter your platform
  provides (`SysTick`, an RTC, a simulation clock, ...).
- **Overflow semantics match the C original on purpose.** Where FastLED
  relies on `u8`/`u16` truncation, signed wraparound, or 32-bit overflow in
  an intermediate product, this port reproduces it exactly (via
  `wrapping_*`/explicit casts) rather than "fixing" it — these functions are
  used as building blocks in animations that depend on the exact periodic
  behavior.
- **`const fn` wherever possible.** Most functions are `const fn`, so lookup
  tables, color palettes, and animation parameters can be computed at compile
  time.

## Verifying against FastLED

This isn't just "should behave like FastLED" — it's checked against FastLED's
actual C reference code on every test run:

- [`fastled-ref/`](fastled-ref/) is a small helper crate that compiles a
  faithful C transcription of FastLED's portable reference algorithms
  ([`shim.c`](fastled-ref/src/shim.c), built via the `cc` crate) and exposes
  them through `extern "C"` bindings. It's a `[dev-dependencies]`-only path
  dependency, so its `unsafe` FFI code never enters `lib8tion` itself —
  `#![forbid(unsafe_code)]` stays intact.
- [`tests/differential.rs`](tests/differential.rs) exhaustively (for `u8`
  domains) or densely (for wider domains) asserts that every ported function
  in `lib8tion` produces **bit-for-bit identical** output to FastLED's C code.
- [`tests/properties.rs`](tests/properties.rs) adds `proptest`-driven
  property tests: randomized differential checks (with shrinking, for the
  domains too wide to sweep exhaustively) plus algebraic invariants the Rust
  port must satisfy on its own (saturation bounds, range bounds, identities at
  fixed points).

```sh
cargo test
```

(`fastled-ref` is a path-only dev-dependency and isn't published — `cargo test`
needs a git clone of this repo, not the crates.io source tarball.)

## `no_std` / embedded

The crate has zero dependencies and builds for bare-metal targets:

```sh
cargo build --target thumbv7em-none-eabihf --release
```

## License

MIT
