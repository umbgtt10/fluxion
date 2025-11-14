// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::fmt::{self, Display};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Animal {
    pub name: String,
    pub legs: u32,
}

impl Animal {
    #[must_use]
    pub const fn new(name: String, legs: u32) -> Self {
        Self { name, legs }
    }
}

impl Display for Animal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Animal[name={}, legs={}]", self.name, self.legs)
    }
}
