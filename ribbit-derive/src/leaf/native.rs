use crate::leaf::Arbitrary;

#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) enum Native {
    N8,
    N16,
    N32,
    N64,
}

impl Native {
    pub(crate) fn size(&self) -> usize {
        match self {
            Self::N8 => 8,
            Self::N16 => 16,
            Self::N32 => 32,
            Self::N64 => 64,
        }
    }

    pub(crate) fn mask(&self) -> usize {
        Arbitrary::new(self.size()).mask()
    }
}
