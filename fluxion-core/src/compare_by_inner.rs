pub trait CompareByInner {
    fn cmp_inner(&self, other: &Self) -> std::cmp::Ordering;
}
