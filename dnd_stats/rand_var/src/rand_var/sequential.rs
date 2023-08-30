use std::{cmp, fmt};
use std::collections::BTreeSet;
use std::fmt::{Debug, Display, Formatter};

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
