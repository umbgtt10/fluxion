use std::fmt::{self, Display};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Person {
    pub name: String,
    pub age: u32,
}

impl Person {
    #[must_use]
    pub const fn new(name: String, age: u32) -> Self {
        Self { name, age }
    }
}

impl Display for Person {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Person[name={}, age={}]", self.name, self.age)
    }
}
