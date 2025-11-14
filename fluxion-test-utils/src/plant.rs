// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

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
