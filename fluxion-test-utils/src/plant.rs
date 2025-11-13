use std::fmt::{self, Display};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Plant {
    pub species: String,
    pub height: u32,
}

impl Plant {
    #[must_use]
    pub const fn new(species: String, height: u32) -> Self {
        Self { species, height }
    }
}

impl Display for Plant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Plant[species={}, height={}]", self.species, self.height)
    }
}
