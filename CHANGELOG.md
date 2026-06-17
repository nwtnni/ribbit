# v0.2.0

- Tweak trait bounds and implementations for `ribbit::Atomic`
    - Relax `T: Pack` bound on generic parameter in struct definition
    - Relax `T: Pack` bound on `Default` and `new_raw` (renamed to `from_raw`)
    - Implement `Clone` and `From<R>` to match standard library atomic types
- Replace `ribbit::Atomic::get_packed` and `ribbit::Atomic::set_packed` with `ribbit::Atomic::get_mut_packed`
- Write documentation for `ribbit::Atomic`

# v0.1.0

Initial release.
