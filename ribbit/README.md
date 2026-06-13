# ribbit

This crate provides a procedural macro (`Pack`) for deriving a
[bit field](https://en.wikipedia.org/wiki/Bit_field) from a Rust type.

## Differentiation

The driving motivation for this crate is lock-free programming,
which often requires packing data into a `u64` or `u128` so it
can be atomically updated. As a result, we don't support
parsing-related functionality, like checked construction of a packed
type from arbitrary bytes, packed types larger than 128 bits,
or non-native endianness. We also don't suport arrays or
anonymous tuples.

This crate does provide the following features that were hard to
find in existing crates:

- Composition of bit fields
- Generic bit fields
- Enums with struct and tuple variants
- `const` inherent methods and constructors (that work with generics, on stable Rust)
- Append-only: does not overwrite original type
- Nonzero support (e.g., can use [`NonZeroU64`] to enable niche optimizations)

See also:
- [bitfield-struct-rs](https://github.com/wrenger/bitfield-struct-rs)
- [bilge](https://github.com/hecatia-elegua/bilge)
- [modular_bitfield](https://github.com/modular-bitfield/modular-bitfield)
- [bitbybit](https://github.com/danlehmann/bitfield)

## Conditional compilation

- The `atomic` feature enables support for atomic operations on packed types.
- The `u128` feature enables support for packed types up to 128 bits instead of 64 bits.
- The `atomic-u128` feature enables support for atomic operations on 128 bit packed types via
  the [`portable-atomic`](https://github.com/taiki-e/portable-atomic) crate.

## Examples

```rust
use core::marker::PhantomData;
use ribbit::u12;
use ribbit::u52;
use ribbit::u31;

// Enums with data
#[derive(ribbit::Pack, Copy, Clone, Debug, PartialEq, Eq)]
#[ribbit(size = 34, derive(Debug, Eq))]
pub enum Enum {
    Unit,
    #[ribbit(size = 16)]
    Tuple(u8, u8),
    #[ribbit(size = 32)]
    Struct {
        a: bool,
        b: u31,
    }
}

// Generic types
#[derive(ribbit::Pack)]
#[ribbit(size = 64)]
pub struct Versioned<T, U> {
    // Configurable method generation
    #[ribbit(with(vis = "pub"))]
    version: u12,
    // Composition of bit fields
    #[ribbit(size = 52, get(vis = "pub"), with(skip))]
    data: T,
    _type: PhantomData<U>,
}

impl<T: Copy, U> Clone for Versioned<T, U> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: Copy, U> Copy for Versioned<T, U> {}

// Non-zero support
use core::num::NonZeroU8;
use ribbit::u3;

#[derive(ribbit::Pack, Copy, Clone)]
#[ribbit(size = 11, non_zero)]
struct NonZero(u3, NonZeroU8);

const _: () = {
    assert!(
        core::mem::size_of::<ribbit::Packed<NonZero>>() ==
        core::mem::size_of::<Option<ribbit::Packed<NonZero>>>(),
    );
    assert!(
        core::mem::align_of::<ribbit::Packed<NonZero>>() ==
        core::mem::align_of::<Option<ribbit::Packed<NonZero>>>()
    );
};

// Const operations on stable
let unit = const {
    ribbit::Packed::<Versioned<Enum, ()>>::new(u12::new(5), ribbit::Packed::<Enum>::new_unit())
        .with_version(u12::new(6))
        .data()
};

use ribbit::Unpack as _;
assert_eq!(unit.unpack(), Enum::Unit);

let tuple = const { ribbit::Packed::<Enum>::new_tuple(12, 15) };
assert_ne!(tuple.unpack(), Enum::Unit);

// Atomic support
use core::sync::atomic::Ordering;
let atomic = ribbit::Atomic::<Enum>::new_packed(tuple);
// Operate on packed types
assert_eq!(
    atomic.compare_exchange_packed(unit, tuple, Ordering::Relaxed, Ordering::Relaxed),
    Err(tuple),
);
// Or on unpacked types
assert_eq!(
    atomic.compare_exchange(tuple.unpack(), unit.unpack(), Ordering::Relaxed, Ordering::Relaxed),
    Ok(tuple.unpack()),
);

assert_eq!(
    atomic.load(Ordering::Relaxed),
    unit.unpack(),
);
```

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
