use std::collections::{BTreeMap, BTreeSet};
use std::iter::Sum;

use num::{BigRational, FromPrimitive, Rational64};

use crate::num_rand_var::NumRandVar;
use crate::rand_var::prob_type::Prob;
use crate::rand_var::RandVar;
use crate::rand_var::sequential::{Pair, SeqIter};
use crate::RVError;
use crate::vec_rand_var::VecRandVar;

#[derive(PartialEq, Clone, Debug)]
pub struct MapRandVar<K: Ord + Clone, P: Prob> {
    lower_bound: K,
    upper_bound: K,
    pdf_map: BTreeMap<K, P>,
}

pub type MRV64 = MapRandVar<isize, Rational64>;
pub type MRVBig = MapRandVar<isize, BigRational>;

impl<K: Ord + Clone, P: Prob> MapRandVar<K, P> {
    pub fn backing_map(&self) -> &BTreeMap<K, P> {
        &self.pdf_map
    }

    pub fn get_lb(&self) -> &K {
        &self.lower_bound
    }

    pub fn get_ub(&self) -> &K {
        &self.upper_bound
    }
}

impl<K, P> MapRandVar<K, P>
where
    K: Ord + Clone,
    P: Prob + PartialOrd<P> + for<'a> Sum<&'a P>,
{
    pub fn from_map(m: BTreeMap<K, P>) -> Result<Self, RVError> {
        if m.len() == 0 {
            return Err(RVError::Other(String::from("map should be non-empty")));
        }
        let lb = m.first_key_value().unwrap().0;
        let ub = m.last_key_value().unwrap().0;
        if lb > ub {
            return Err(RVError::InvalidBounds);
        }
        if m.values().sum::<P>() != P::one() {
            return Err(RVError::CDFNotOne);
        }
        if m.values().any(|p| p < &P::zero()) {
            return Err(RVError::NegProb);
        }
        Ok(Self {
            lower_bound: lb.clone(),
            upper_bound: ub.clone(),
            pdf_map: m
        })
    }

    pub fn independent_trials_self(&self) -> MapRandVar<Pair<K, K>, P> {
        self.independent_trials(&self)
    }

    pub fn independent_trials<L>(&self, other: &MapRandVar<L, P>) -> MapRandVar<Pair<K, L>, P>
    where
        L: Ord + Clone
    {
        let mut new_pdf: BTreeMap<Pair<K, L>, P> = BTreeMap::new();
        for k in self.get_keys() {
            for l in other.get_keys() {
                let val = self.pdf_ref(&k) * other.pdf_ref(&l);
                new_pdf.insert(Pair(k.clone(), l), val);
            }
        }
        // .unwrap() is fine here, because if self is a valid RV, then this also will be.
        MapRandVar::from_map(new_pdf).unwrap()
    }

    pub fn map_keys<L, F>(&self, f: F) -> MapRandVar<L, P>
    where
        L: Ord + Clone,
        F: Fn(K) -> L,
    {
        let mut new_pdf: BTreeMap<L, P> = BTreeMap::new();
        for k in self.get_keys() {
            let l = f(k.clone());
            if new_pdf.contains_key(&l) {
                let old_t = new_pdf.remove(&l).unwrap();
                new_pdf.insert(l, old_t + self.pdf_ref(&k));
            } else {
                new_pdf.insert(l, self.pdf_ref(&k));
            }
        }
        // .unwrap() is fine here, because if self is a valid RV, then this also will be.
        MapRandVar::from_map(new_pdf).unwrap()
    }

    pub fn consolidate<L, RV>(&self, outcomes: &BTreeMap<K, RV>) -> Result<MapRandVar<L, P>, RVError>
    where
        L: Ord + Clone,
        RV: RandVar<L, P>,
    {
        let mut new_pdf: BTreeMap<L, P> = BTreeMap::new();
        for k in self.get_keys() {
            match outcomes.get(&k) {
                None => return Err(RVError::Other(String::from("every valid k must have an outcome"))),
                Some(rv) => {
                   for l in rv.get_keys() {
                       let pdf_kl = self.pdf_ref(&k) * rv.pdf_ref(&l);
                       if pdf_kl > P::zero() {
                           new_pdf.entry(l)
                               .and_modify(|p| *p = p.clone() + pdf_kl.clone())
                               .or_insert(pdf_kl);
                       }
                   }
                }
            }
        }
        MapRandVar::from_map(new_pdf)
    }

    pub fn projection<L, RV>(&self, outcomes: &BTreeMap<K, RV>) -> Result<MapRandVar<Pair<K, L>, P>, RVError>
    where
        L: Ord + Clone,
        RV: RandVar<L, P>,
    {
        let mut new_pdf: BTreeMap<Pair<K, L>, P> = BTreeMap::new();
        for k in self.get_keys() {
            match outcomes.get(&k) {
                None => return Err(RVError::Other(String::from("every valid k must have an outcome!"))),
                Some(rv) => {
                    for l in rv.get_keys() {
                        let pdf_kl = self.pdf_ref(&k) * rv.pdf_ref(&l);
                        if pdf_kl > P::zero() {
                            new_pdf.insert(Pair(k.clone(), l), pdf_kl);
                        }
                    }
                }
            }
        }
        MapRandVar::from_map(new_pdf)
    }
}

impl<K, P> RandVar<K, P> for MapRandVar<K, P>
where
    K: Ord + Clone,
    P: Prob + PartialOrd<P>,
{
    fn build<F: Fn(K) -> P>(seq_iter: SeqIter<K>, f: F) -> Result<Self, RVError> {
        if seq_iter.items.len() == 0 {
            return Err(RVError::InvalidBounds);
        }
        let lb = seq_iter.items.first().unwrap().clone();
        let ub = seq_iter.items.last().unwrap().clone();
        let mut pdf_map = BTreeMap::new();
        let mut total = P::zero();
        for k in seq_iter {
            let f_k = f(k.clone());
            if f_k < P::zero() {
                return Err(RVError::NegProb);
            }
            if f_k > P::zero() {
                total = total + f_k.clone();
                pdf_map.insert(k, f_k);
            }
        }
        if P::one() != total {
            return Err(RVError::CDFNotOne);
        }

        Ok(Self {
            lower_bound: lb,
            upper_bound: ub,
            pdf_map
        })
    }

    fn lower_bound(&self) -> K {
        self.lower_bound.clone()
    }

    fn upper_bound(&self) -> K {
        self.upper_bound.clone()
    }

    unsafe fn raw_pdf(&self, k: &K) -> P {
        if let Some(p) = self.pdf_map.get(k) {
            p.clone()
        } else {
            P::zero()
        }
    }

    fn get_keys(&self) -> SeqIter<K> {
        let items: BTreeSet<K> = self.pdf_map.keys().cloned().collect();
        SeqIter { items }
    }

    fn len(&self) -> usize {
        self.pdf_map.len()
    }
}

impl<P> NumRandVar<isize, P> for MapRandVar<isize, P>
where
    P: Prob + PartialOrd<P> + FromPrimitive
{
    fn convert(&self, k: isize) -> P {
        P::from_isize(k).unwrap()
    }
}

impl<K: Ord + Clone, P: Prob> From<MapRandVar<K, P>> for BTreeMap<K, P> {
    fn from(value: MapRandVar<K, P>) -> Self {
        value.pdf_map
    }
}

impl<P: Prob + PartialOrd<P>> From<VecRandVar<P>> for MapRandVar<isize, P> {
    fn from(value: VecRandVar<P>) -> Self {
        let lb = value.lower_bound();
        let ub = value.upper_bound();
        let mut pdf_map: BTreeMap<isize, P> = BTreeMap::new();
        let pdf_vec: Vec<P> = value.into();
        for (i, p) in pdf_vec.into_iter().enumerate() {
            if p > P::zero() {
                pdf_map.insert((i as isize) + lb, p);
            }
        }
        // trust that value is a valid RandomVariable,
        // so this will be a valid MapRandVar.
        Self {
            lower_bound: lb,
            upper_bound: ub,
            pdf_map
        }
    }
}

impl<P: Prob> MapRandVar<isize, P> {
    pub fn into_vrv(self) -> VecRandVar<P> {
        self.into()
    }
}

#[cfg(test)]
mod tests {
    use std::cmp;
    use std::collections::BTreeMap;
    use num::{BigInt, BigRational, FromPrimitive, One, Rational64};
    use crate::map_rand_var::{MapRandVar, MRV64};
    use crate::num_rand_var::NumRandVar;
    use crate::rand_var::RandVar;
    use crate::rand_var::sequential::{Nested, Pair};
    use crate::vec_rand_var::{VecRandVar, VRV64, VRVBig};

    fn get_attack_setup() -> (MRV64, BTreeMap<isize, VRV64>) {
        let d20: VRV64 = VecRandVar::new_dice(20).unwrap();
        let hit_bonus= 8;
        let attack_check = d20
            .add_const(hit_bonus)
            .into_mrv()
            .map_keys(|to_hit| {
                // in "real" code, this would return an enum (crit, hit, miss)
                if to_hit == 28 { // natural 20
                    2
                } else if to_hit >= 17 { // AC 17
                    1
                } else {
                    0
                }
            });
        let dmg_bonus = 5;
        let dmg_dice: VRV64 = VecRandVar::new_dice(6).unwrap().multiple(2);

        let hit_dmg = dmg_dice.add_const(dmg_bonus);
        let crit_dmg = dmg_dice.multiple(2).add_const(dmg_bonus);
        let miss_dmg: VRV64 = VecRandVar::new_constant(0).unwrap();

        let mut outcomes: BTreeMap<isize, VRV64> = BTreeMap::new();
        outcomes.insert(0, miss_dmg);
        outcomes.insert(1, hit_dmg);
        outcomes.insert(2, crit_dmg);
        (attack_check, outcomes)
    }

    #[test]
    fn test_consolidate() {
        let (attack_check, outcomes) = get_attack_setup();
        let attack_dmg = attack_check.consolidate(&outcomes).unwrap();
        assert_eq!(0, attack_dmg.lower_bound());
        assert_eq!(29, attack_dmg.upper_bound());
        assert_eq!(Rational64::new(8, 20), attack_dmg.pdf(0));
        assert_eq!(Rational64::new(1, 25920), attack_dmg.pdf(29));
        assert_eq!(Rational64::new(151, 20), attack_dmg.expected_value());
    }

    #[test]
    fn test_projection() {
        let (attack_check, outcomes) = get_attack_setup();
        let attack_outcome = attack_check.projection(&outcomes).unwrap();
        assert_eq!(Pair(0,0), attack_outcome.lower_bound());
        assert_eq!(Pair(2, 29), attack_outcome.upper_bound());
        assert_eq!(Rational64::new(8, 20), attack_outcome.pdf(Pair(0,0)));
        assert_eq!(Rational64::new(1, 25920), attack_outcome.pdf(Pair(2, 29)));
        assert_eq!(Rational64::new(151, 20), attack_outcome.general_expected_value(|pair| Rational64::from_isize(pair.1).unwrap()));

        let attack_dmg = attack_check.consolidate(&outcomes).unwrap();
        assert_eq!(attack_dmg, attack_outcome.map_keys(|pair| pair.1));
    }

    #[test]
    fn test_4d6_drop_lowest() {
        let d6: VRVBig = VecRandVar::new_dice(6).unwrap();
        let ability_score = d6
            .into_mrv()
            .independent_trials_self()
            .independent_trials_self()
            .map_keys(|rolls| {
                let smallest = cmp::min(
                    cmp::min(rolls.0.0, rolls.0.1),
                    cmp::min(rolls.1.0, rolls.1.1));
                rolls.0.0 + rolls.0.1 + rolls.1.0 + rolls.1.1 - smallest
            }).into_vrv();
        assert_eq!(3, ability_score.lower_bound());
        assert_eq!(18, ability_score.upper_bound());
        assert_eq!(BigRational::new(BigInt::from_isize(15869).unwrap(), BigInt::from_isize(1296).unwrap()), ability_score.expected_value());

        let ability_score2 = VecRandVar::new_dice(6).unwrap()
            .into_mrv()
            .independent_trials_self()
            .independent_trials_self()
            .map_keys(|rolls| {
                rolls.flat_sum() - rolls.flat_min()
            }).into_vrv();
        assert_eq!(ability_score, ability_score2);
    }

    #[test]
    fn test_super_adv_two_ways() {
        let d20_super_adv: VRV64 = VecRandVar::new_dice(20).unwrap().max_three_trials();
        let other_d20: MRV64 = VecRandVar::new_dice(20).unwrap().into();
        let other_d20_super_adv: VRV64 = other_d20
            .independent_trials(&other_d20)
            .independent_trials(&other_d20)
            .map_keys(|triple| triple.flat_max())
            .into();
        assert_eq!(d20_super_adv, other_d20_super_adv);
    }

    #[test]
    fn test_adv_two_ways() {
        let d20_adv: VRV64 = VecRandVar::new_dice(20).unwrap().max_two_trials();
        let other_d20: MRV64 = VecRandVar::new_dice(20).unwrap().into();
        let other_d20_adv: VRV64 = other_d20
            .independent_trials(&other_d20)
            .map_keys(|pair| cmp::max(pair.0, pair.1))
            .into();
        assert_eq!(d20_adv, other_d20_adv);
    }

    #[test]
    fn test_2d6_two_ways() {
        let two_d6: VRV64 = VecRandVar::new_dice(6).unwrap().multiple(2);
        let other_d6: MRV64 = VecRandVar::new_dice(6).unwrap().into();
        let other_2d6: VRV64 = other_d6
            .independent_trials(&other_d6)
            .map_keys(|pair| pair.0 + pair.1)
            .into();
        assert_eq!(two_d6, other_2d6);
    }

    #[test]
    fn test_ad_hoc() {
        let mut pdf: BTreeMap<isize, Rational64> = BTreeMap::new();
        pdf.insert(0, Rational64::new(1, 3));
        pdf.insert(3, Rational64::new(1, 9));
        pdf.insert(10, Rational64::new(1, 5));
        pdf.insert(25, Rational64::new(1, 4));
        pdf.insert(33, Rational64::new(1, 10));
        pdf.insert(42, Rational64::new(1, 180));

        let rv = MapRandVar::from_map(pdf).unwrap();
        assert_eq!(0, rv.lower_bound());
        assert_eq!(42, rv.upper_bound());
        assert_eq!(Rational64::one(), rv.cdf(42));
        assert_eq!(Rational64::new(179, 180), rv.cdf(37));
        assert_eq!(Rational64::new(727,60), rv.expected_value());
    }
}
