// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

use core::fmt::{self, Display};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Animal {
    pub species: String,
    pub legs: u32,
}

impl Animal {
    #[must_use]
    pub const fn new(species: String, legs: u32) -> Self {
        Self { species, legs }
    }
}

impl Display for Animal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Animal[species={}, legs={}]", self.species, self.legs)
    }
}
