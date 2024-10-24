use crate::repr::Native;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct Arbitrary {
    size: usize,
}

impl Arbitrary {
    pub(super) fn new(size: usize) -> Self {
        Self { size }
    }

    pub(crate) fn size(&self) -> usize {
        self.size
    }

    pub(crate) fn mask(&self) -> usize {
        1usize
            .checked_shl(self.size as u32)
            .and_then(|mask| mask.checked_sub(1))
            .unwrap_or(usize::MAX)
    }

    pub(crate) fn as_native(&self) -> Native {
        match self.size {
            0..=7 => Native::N8,
            9..=15 => Native::N16,
            17..=31 => Native::N32,
            33..=63 => Native::N64,
            _ => unreachable!(),
        }
    }
}
