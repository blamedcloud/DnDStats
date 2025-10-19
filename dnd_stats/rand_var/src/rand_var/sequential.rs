use std::{cmp, fmt};
use std::collections::BTreeSet;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Add;
use num::Rational64;

#[derive(Clone)]
pub struct SeqIter<K: Ord + Clone> {
    pub items: BTreeSet<K>,
}

impl<K: Ord + Clone> Iterator for SeqIter<K> {
    type Item = K;

    fn next(&mut self) -> Option<Self::Item> {
        self.items.pop_first()
    }
}

pub trait Seq
where
    Self: Ord + Clone + Sized
{
    fn gen_seq(&self, other: &Self) -> SeqIter<Self>;
    fn always_convex() -> bool;
    fn convex_bounds(iter: SeqIter<Self>) -> Option<(Self, Self)>;
}

pub fn always_convex_bounds<K>(iter: SeqIter<K>) -> Option<(K, K)>
where
    K: Ord + Clone
{
    if iter.items.len() == 0 {
        None
    } else {
        Some((iter.items.first().unwrap().clone(), iter.items.last().unwrap().clone()))
    }
}

impl Seq for isize {
    fn gen_seq(&self, other: &Self) -> SeqIter<Self> {
        let first = *cmp::min(self, other);
        let second = *cmp::max(self, other);
        let items = BTreeSet::from_iter(first..=second);
        SeqIter { items }
    }

    fn always_convex() -> bool {
        true
    }

    fn convex_bounds(iter: SeqIter<Self>) -> Option<(Self, Self)> {
        always_convex_bounds(iter)
    }
}

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Clone)]
pub struct Pair<A: Ord + Clone, B: Ord + Clone>(pub A, pub B);

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum FlatType {
    Max,
    Min,
    Sum,
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum FlatCmp {
    Max,
    Min,
}

pub trait Nested<K: Ord + Clone> {

    fn get_k(&self) -> K;
    fn flat_max(&self) -> K { self.get_k() }
    fn flat_min(&self) -> K { self.get_k() }

    fn flat_sum(&self) -> K
    where K: Add<K, Output=K>
    {
        self.get_k()
    }

    fn flat_cmp(&self, fc: FlatCmp) -> K {
        match fc {
            FlatCmp::Max => self.flat_max(),
            FlatCmp::Min => self.flat_min(),
        }
    }

    fn flatten(&self, ft: FlatType) -> K
    where K: Add<K, Output=K>
    {
        match ft {
            FlatType::Max => self.flat_max(),
            FlatType::Min => self.flat_min(),
            FlatType::Sum => self.flat_sum(),
        }
    }
}

impl<K, A, B> Nested<K> for Pair<A, B>
where
    K: Ord + Clone,
    A: Nested<K> + Ord + Clone,
    B: Nested<K> + Ord + Clone
{
    // This method should never be called because a Pair is not a K.
    // One could do something like get the first K in the nested Pairs
    // if they desperately wanted this to not panic.
    // I'm fine with this the way it is, because if any of the flat_*
    // methods accidentally call this, it will loudly fail, rather
    // than silently return the wrong thing.
    fn get_k(&self) -> K {
        unimplemented!()
    }

    fn flat_max(&self) -> K {
        let left = self.0.flat_max();
        let right = self.1.flat_max();
        cmp::max(left, right)
    }

    fn flat_min(&self) -> K {
        let left = self.0.flat_min();
        let right = self.1.flat_min();
        cmp::min(left, right)
    }

    fn flat_sum(&self) -> K
    where
        K: Add<K, Output=K>
    {
        let left = self.0.flat_sum();
        let right = self.1.flat_sum();
        left + right
    }
}

impl Nested<isize> for isize {
    fn get_k(&self) -> isize {
        *self
    }
}

impl Nested<Rational64> for Rational64 {
    fn get_k(&self) -> Rational64 {
        *self
    }
}

impl<A, B> Seq for Pair<A, B>
where
    A: Seq + Clone,
    B: Seq + Clone,
{
    // care must be taken here, because this function
    // won't always generate every Pair<A, B> between
    // self and other. For example:
    // get_seq(Pair(0,0), Pair(10,5)) will not generate
    // Pair(6,7) even though Pair(0,0) < Pair(6,7) < Pair(10,5)
    fn gen_seq(&self, other: &Self) -> SeqIter<Self> {
        let a_sg = A::gen_seq(&self.0, &other.0);
        let b_sg = B::gen_seq(&self.1, &other.1);
        let ab_set: BTreeSet<Pair<A, B>> = itertools::iproduct!(a_sg, b_sg).map(|(a, b)| Pair(a, b)).collect();
        SeqIter { items: ab_set }
    }

    fn always_convex() -> bool {
        false
    }

    fn convex_bounds(iter: SeqIter<Self>) -> Option<(Self, Self)> {
        if iter.items.len() == 0 {
            None
        } else {
            let mut min_a = iter.items.first().unwrap().0.clone();
            let mut min_b = iter.items.first().unwrap().1.clone();
            let mut max_a = iter.items.last().unwrap().0.clone();
            let mut max_b = iter.items.last().unwrap().1.clone();
            for Pair(a, b) in iter {
                min_a = cmp::min(min_a, a.clone());
                max_a = cmp::max(max_a, a);
                min_b = cmp::min(min_b, b.clone());
                max_b = cmp::max(max_b, b);
            }
            Some((Pair(min_a, min_b), Pair(max_a, max_b)))
        }
    }
}

impl<A, B> Display for Pair<A, B>
where
    A: Ord + Clone + Display,
    B: Ord + Clone + Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.0, self.1)
    }
}

impl<S> Seq for Option<S>
where
    S: Seq + Clone + Default
{
    fn gen_seq(&self, other: &Self) -> SeqIter<Self> {
        let mut set: BTreeSet<Option<S>> = BTreeSet::new();
        match (self, other) {
            (None, None) => {},
            (None, Some(s)) => {
                set = S::gen_seq(&S::default(), s).map(|x| Some(x)).collect();
            },
            (Some(s), None) => {
                set = S::gen_seq(&S::default(), s).map(|x| Some(x)).collect();
            },
            (Some(s1), Some(s2)) => set = S::gen_seq(s1, s2).map(|x| Some(x)).collect(),
        };
        set.insert(None);
        SeqIter { items: set }
    }

    fn always_convex() -> bool {
        false
    }

    fn convex_bounds(iter: SeqIter<Self>) -> Option<(Self, Self)> {
        let set: BTreeSet<S> = iter.map(|os| os.unwrap_or(S::default())).collect();
        S::convex_bounds(SeqIter { items: set }).map(|(min, max)| (Some(min), Some(max)))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use num::Rational64;
    use crate::rand_var::sequential::{FlatCmp, FlatType, Nested, Pair, Seq};

    #[test]
    fn test_isize() {
        let seq_iter = isize::gen_seq(&3, &7);
        assert_eq!(5, seq_iter.items.len());
        let items = BTreeSet::from_iter(3..=7);
        assert_eq!(items, seq_iter.items);

        let bounds = isize::convex_bounds(seq_iter);
        assert!(bounds.is_some());
        let (lb, ub) = bounds.unwrap();
        assert_eq!(3, lb);
        assert_eq!(7, ub);
    }

    #[test]
    fn test_pair() {
        let pair1 = Pair::<isize, isize>(2, 5);
        let pair2 = Pair::<isize, isize>(-1, 7);
        let seq_iter = pair1.gen_seq(&pair2);
        assert_eq!(12, seq_iter.items.len());

        let mut items = BTreeSet::new();
        items.insert(Pair(-1, 5));
        items.insert(Pair(-1, 6));
        items.insert(Pair(-1, 7));
        items.insert(Pair(0, 5));
        items.insert(Pair(0, 6));
        items.insert(Pair(0, 7));
        items.insert(Pair(1, 5));
        items.insert(Pair(1, 6));
        items.insert(Pair(1, 7));
        items.insert(Pair(2, 5));
        items.insert(Pair(2, 6));
        items.insert(Pair(2, 7));

        assert_eq!(items, seq_iter.items);

        let bounds = Pair::<isize, isize>::convex_bounds(seq_iter);
        assert!(bounds.is_some());
        let (lb, ub) = bounds.unwrap();
        assert_eq!(Pair(-1, 5), lb);
        assert_eq!(Pair(2, 7), ub);
    }

    #[test]
    fn test_flatten() {
        let p1 = Pair(0, Pair(-1, 1));
        assert_eq!(1, p1.flat_max());
        assert_eq!(1, p1.flatten(FlatType::Max));
        assert_eq!(1, p1.flat_cmp(FlatCmp::Max));
        assert_eq!(-1, p1.flat_min());
        assert_eq!(-1, p1.flatten(FlatType::Min));
        assert_eq!(-1, p1.flat_cmp(FlatCmp::Min));
        assert_eq!(0, p1.flat_sum());
        assert_eq!(0, p1.flatten(FlatType::Sum));

        let p2 = Pair(Pair(Pair(3, Pair(0, 5)), Pair(1, 7)), Pair(Pair(2, Pair(Pair(Pair(4, Pair(-1, 6)), 0), Pair(-2, 8))), Pair(-3, 9)));
        assert_eq!(9, p2.flat_max());
        assert_eq!(9, p2.flatten(FlatType::Max));
        assert_eq!(9, p2.flat_cmp(FlatCmp::Max));
        assert_eq!(-3, p2.flat_min());
        assert_eq!(-3, p2.flatten(FlatType::Min));
        assert_eq!(-3, p2.flat_cmp(FlatCmp::Min));
        assert_eq!(39, p2.flat_sum());
        assert_eq!(39, p2.flatten(FlatType::Sum));

        let p3 = Pair(Rational64::from_integer(2), Pair(Rational64::from_integer(3), Rational64::from_integer(1)));
        assert_eq!(Rational64::from_integer(3), p3.flat_max());
        assert_eq!(Rational64::from_integer(1), p3.flat_min());
        assert_eq!(Rational64::from_integer(6), p3.flat_sum());
    }
}
