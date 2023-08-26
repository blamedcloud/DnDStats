

pub trait Transposition {
    fn is_transposition(&self, other: &Self) -> bool;
    fn merge_left(&mut self, other: Self);
}
