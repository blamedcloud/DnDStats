use std::collections::{BTreeMap, BTreeSet};
use std::marker::PhantomData;
use std::ops::Add;

use crate::rand_var::prob_type::{Prob, Reciprocal};
use crate::rand_var::RandVar;
use crate::rand_var::sequential::SeqIter;

pub struct RVPartition<K: Ord + Clone, P: Prob, RV: RandVar<K, P>> {
    pub prob: P,
    pub rv: Option<RV>,
    _k: PhantomData<K>,
}

impl<K: Ord + Clone, P: Prob, RV: RandVar<K, P>> RVPartition<K, P, RV> {
    pub fn new(prob: P, rv: RV) -> Self {
        Self {
            prob,
            rv: Some(rv),
            _k: PhantomData,
        }
    }

    pub fn empty() -> Self {
        Self {
            prob: P::zero(),
            rv: None,
            _k: PhantomData
        }
    }
}

impl<K: Ord + Clone, P: Prob + Reciprocal, RV: RandVar<K, P>> Add for RVPartition<K, P, RV> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        if rhs.rv.is_none() {
            return self;
        }
        if self.rv.is_none() {
            return rhs;
        }
        let new_prob = self.prob.clone() + rhs.prob.clone();
        let new_prob_recip = new_prob.clone().reciprocal().unwrap();

        let left_mult = self.prob * new_prob_recip.clone();
        let left_rv = self.rv.unwrap();

        let right_mult = rhs.prob * new_prob_recip;
        let right_rv = rhs.rv.unwrap();

        let mut all_k = BTreeSet::new();
        let mut pdf_map: BTreeMap<K, P> = BTreeMap::new();

        for k in left_rv.get_keys() {
            all_k.insert(k.clone());
            let p = left_mult.clone() * left_rv.pdf_ref(&k);
            if pdf_map.contains_key(&k) {
                let old_p = pdf_map.get(&k).unwrap().clone();
                pdf_map.insert(k, old_p + p);
            } else {
                pdf_map.insert(k, p);
            }
        }

        for k in right_rv.get_keys() {
            all_k.insert(k.clone());
            let p = right_mult.clone() * right_rv.pdf_ref(&k);
            if pdf_map.contains_key(&k) {
                let old_p = pdf_map.get(&k).unwrap().clone();
                pdf_map.insert(k, old_p + p);
            } else {
                pdf_map.insert(k, p);
            }
        }

        let all_k_si = SeqIter { items: all_k };
        let new_rv = RandVar::build(all_k_si, |k| {
            if pdf_map.contains_key(&k) {
                pdf_map.get(&k).unwrap().clone()
            } else {
                P::zero()
            }
        }).unwrap();
        RVPartition::new(new_prob, new_rv)
    }
}

#[cfg(test)]
mod tests {
    use num::{One, Rational64};
    use crate::rand_var::RandVar;
    use crate::vec_rand_var::{VecRandVar, VRV64};

    #[test]
    fn test_partitions() {
        let d20: VRV64 = VecRandVar::new_dice(20).unwrap();
        let mut mod_3 = d20.partitions(|p| *p % 3);
        assert_eq!(3, mod_3.len());
        let part_0 = mod_3.remove(&0).unwrap();
        assert_eq!(Rational64::new(6, 20), part_0.prob);
        assert_eq!(3, part_0.rv.as_ref().unwrap().lower_bound());
        assert_eq!(18, part_0.rv.as_ref().unwrap().upper_bound());

        let part_1 = mod_3.remove(&1).unwrap();
        assert_eq!(Rational64::new(7, 20), part_1.prob);
        assert_eq!(1, part_1.rv.as_ref().unwrap().lower_bound());
        assert_eq!(19, part_1.rv.as_ref().unwrap().upper_bound());

        let part_2 = mod_3.remove(&2).unwrap();
        assert_eq!(Rational64::new(7, 20), part_2.prob);
        assert_eq!(2, part_2.rv.as_ref().unwrap().lower_bound());
        assert_eq!(20, part_2.rv.as_ref().unwrap().upper_bound());

        let part_0_1 = part_0 + part_1;
        assert_eq!(Rational64::new(13, 20), part_0_1.prob);
        assert_eq!(1, part_0_1.rv.as_ref().unwrap().lower_bound());
        assert_eq!(19, part_0_1.rv.as_ref().unwrap().upper_bound());

        let part_all = part_0_1 + part_2;
        assert_eq!(Rational64::one(), part_all.prob);
        assert_eq!(d20, part_all.rv.unwrap());
    }
}
