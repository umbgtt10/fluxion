use std::cmp::Ordering;

pub trait CompareByInner {
    fn cmp_inner(&self, other: &Self) -> Ordering;
}
