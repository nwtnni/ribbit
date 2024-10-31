use crate::ty::Loose;

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

    pub(crate) fn loosen(&self) -> Loose {
        match self.size {
            0..=7 => Loose::N8,
            9..=15 => Loose::N16,
            17..=31 => Loose::N32,
            33..=63 => Loose::N64,
            _ => unreachable!(),
        }
    }
}
