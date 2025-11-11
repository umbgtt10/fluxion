use std::fmt::{self, Display};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Animal {
    pub name: String,
    pub legs: u32,
}

impl Animal {
    pub fn new(name: String, legs: u32) -> Self {
        Self { name, legs }
    }
}

impl Display for Animal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Animal[name={}, legs={}]", self.name, self.legs)
    }
}
