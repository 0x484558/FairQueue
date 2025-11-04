/// Trait for defining grouping logic for fair scheduling structures.
/// Values belong to the same group when `is_same_group` returns true.
pub trait FairGroup {
    fn is_same_group(&self, other: &Self) -> bool;
}
