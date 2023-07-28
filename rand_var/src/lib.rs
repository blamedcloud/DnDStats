use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Debug, Display};
use std::iter::Sum;
use num::{FromPrimitive, Integer, Num, One, Zero};
use num::rational::Ratio;
use crate::rv_traits::{NumRandVar, Pair, RandVar, RVError, Seq, SeqGen};

pub mod rv_traits;

#[derive(PartialEq, Clone, Debug)]
pub struct RandomVariable<T: Num> {
    lower_bound: isize,
    upper_bound: isize,
    pdf_vec: Vec<T>,
}

impl<T> RandomVariable<Ratio<T>>
where
    T: Integer + Debug + Clone + Display + FromPrimitive
{
    pub fn new(lb: isize, ub: isize, v: Vec<Ratio<T>>) -> Result<Self, RVError>
    {
        if lb > ub {
            return Err(RVError::InvalidBounds);
        }
        if (ub-lb+1) != (v.len() as isize) {
            return Err(RVError::Other(String::from("vector must be of correct length")));
        }
        if Ratio::one() != v.iter().sum() {
            return Err(RVError::CDFNotOne);
        }
        if v.iter().min().unwrap() < &Ratio::zero() {
            return Err(RVError::NegProb);
        }
        Ok(RandomVariable {
            lower_bound: lb,
            upper_bound: ub,
            pdf_vec: v
        })
    }

    pub fn new_dice(sides: isize) -> Result<Self, RVError> {
        if sides < 1 {
            return Err(RVError::InvalidBounds);
        }
        <RandomVariable<Ratio<T>> as RandVar<isize, Ratio<T>>>::build(
            1,
            sides,
            |_| {
                Ratio::new(T::one(), T::from_isize(sides).unwrap())
            })
    }

    pub fn new_dice_reroll(sides: isize, reroll_max: isize) -> Result<Self, RVError> {
        if sides < 1 {
            return Err(RVError::InvalidBounds);
        }
        if reroll_max < 1 {
            return Err(RVError::Other(String::from("reroll_max must be positive")));
        }
        if sides <= reroll_max {
            return Err(RVError::Other(String::from("sides must be larger than reroll_max")));
        }
        let one_over_sides = Ratio::new(
            T::one(),
            T::from_isize(sides).unwrap());
        let reroll_over_sides = Ratio::new(
            T::from_isize(reroll_max).unwrap(),
            T::from_isize(sides).unwrap());
        let reroll_only = reroll_over_sides * one_over_sides.clone();
        let possible_reroll = one_over_sides + reroll_only.clone();
        <RandomVariable<Ratio<T>> as RandVar<isize, Ratio<T>>>::build(
            1,
            sides,
            |x| {
                if x > reroll_max {
                    possible_reroll.clone()
                } else {
                    reroll_only.clone()
                }
            })
    }

    pub fn new_constant(value: isize) -> Result<Self, RVError> {
        <RandomVariable<Ratio<T>> as RandVar<isize, Ratio<T>>>::build(
            value,
            value,
            |_| Ratio::one())
    }

    pub fn new_uniform(lb: isize, ub: isize) -> Result<Self, RVError> {
        <RandomVariable<Ratio<T>> as RandVar<isize, Ratio<T>>>::build(
            lb,
            ub,
            |_| {
                Ratio::new(T::one(), T::from_isize(ub-lb+1).unwrap())
            })
    }

    pub fn to_map_rv(&self) -> MapRandVar<isize, Ratio<T>>
    {
        let mut pdf_map: BTreeMap<isize, Ratio<T>> = BTreeMap::new();
        for (i, t) in self.pdf_vec.iter().enumerate() {
            if t > &Ratio::zero() {
                pdf_map.insert((i as isize)+self.lower_bound, t.clone());
            }
        }
        // .unwrap() is fine here, because if self is a valid RV, then this also will be.
        MapRandVar::from_map(pdf_map).unwrap()
    }
}

impl<T> RandVar<isize, T> for RandomVariable<T>
where
    T: Num + Sum + Debug + Clone + Display + PartialOrd<T>
{
    fn build<F>(lb: isize, ub: isize, f: F) -> Result<RandomVariable<T>, RVError>
    where
        F: Fn(isize) -> T
    {
        if lb > ub {
            return Err(RVError::InvalidBounds);
        }
        let length: usize = (ub - lb + 1) as usize;
        let mut pdf_vec = Vec::with_capacity(length);
        let mut total = T::zero();
        for i in Seq::gen_seq(&lb, &ub) {
            let f_i = f(i);
            if f_i < T::zero() {
                return Err(RVError::NegProb);
            }
            total = total + f_i.clone();
            pdf_vec.push(f_i);
        }
        if T::one() != total {
            return Err(RVError::CDFNotOne);
        }

        Ok(RandomVariable {
            lower_bound: lb,
            upper_bound: ub,
            pdf_vec
        })
    }

    fn lower_bound(&self) -> isize {
        self.lower_bound
    }

    fn upper_bound(&self) -> isize {
        self.upper_bound
    }

    unsafe fn raw_pdf(&self, x: &isize) -> T {
        assert!(self.lower_bound <= *x);
        assert!(*x <= self.upper_bound);
        let index: usize = (x - self.lower_bound) as usize;
        self.pdf_vec.get(index).unwrap().clone()
    }

    fn valid_p(&self) -> SeqGen<isize> {
        Seq::gen_seq(&self.lower_bound(), &self.upper_bound())
    }
}

impl<T> NumRandVar<isize, T> for RandomVariable<T>
where
    T: Num + Sum + Debug + Clone + Display + FromPrimitive + PartialOrd<T>
{
    fn convert(&self, p: isize) -> T {
        T::from_isize(p).unwrap()
    }
}

// BEGIN map rand var //

pub struct MapRandVar<P: Ord, T: Num> {
    lower_bound: P,
    upper_bound: P,
    pdf_map: BTreeMap<P,T>,
}

impl<T> MapRandVar<isize, Ratio<T>>
where
    T: Integer + Debug + Clone + Display + FromPrimitive,
{
    pub fn to_vec_rv(&self) -> RandomVariable<Ratio<T>>
    {
        let lb = self.lower_bound;
        let ub = self.upper_bound;
        let mut pdf_vec = Vec::with_capacity((ub-lb+1) as usize);
        for i in Seq::gen_seq(&lb, &ub) {
            if let Some(t) = self.pdf_map.get(&i) {
                pdf_vec.push(t.clone());
            } else {
                pdf_vec.push(Ratio::zero());
            }
        }
        // .unwrap() is fine here, because if self is a valid RV, then this also will be.
        RandomVariable::new(lb, ub, pdf_vec).unwrap()
    }
}

impl<P, T> MapRandVar<P, T>
where
    P: Ord + Clone + Display + Seq,
    T: Num + Sum + Clone + Display + Debug + PartialOrd<T> + for<'a> Sum<&'a T>,
{
    pub fn from_map(m: BTreeMap<P, T>) -> Result<Self, RVError> {
        if m.len() == 0 {
            return Err(RVError::Other(String::from("map should be non-empty")));
        }
        let lb = m.first_key_value().unwrap().0;
        let ub = m.last_key_value().unwrap().0;
        if lb > ub {
            return Err(RVError::InvalidBounds);
        }
        if m.values().sum::<T>() != T::one() {
            return Err(RVError::CDFNotOne);
        }
        if m.values().any(|t| t < &T::zero()) {
            return Err(RVError::NegProb);
        }
        Ok(MapRandVar {
            lower_bound: lb.clone(),
            upper_bound: ub.clone(),
            pdf_map: m
        })
    }

    pub fn independent_trials_self(&self) -> MapRandVar<Pair<P, P>, T> {
        self.independent_trials(&self)
    }

    pub fn independent_trials<Q>(&self, other: &MapRandVar<Q, T>) -> MapRandVar<Pair<P, Q>, T>
    where
        Q: Ord + Clone + Display + Seq
    {
        let mut new_pdf: BTreeMap<Pair<P, Q>, T> = BTreeMap::new();
        for p in self.valid_p() {
            for q in other.valid_p() {
                let val = self.pdf_ref(&p) * other.pdf_ref(&q);
                new_pdf.insert(Pair(p.clone(), q), val);
            }
        }
        // .unwrap() is fine here, because if self is a valid RV, then this also will be.
        MapRandVar::from_map(new_pdf).unwrap()
    }

    pub fn map_keys<Q, F>(&self, f: F) -> MapRandVar<Q, T>
    where
        Q: Ord + Clone + Display + Seq,
        F: Fn(P) -> Q,
    {
        let mut new_pdf: BTreeMap<Q, T> = BTreeMap::new();
        for p in self.valid_p() {
            let q = f(p.clone());
            if new_pdf.contains_key(&q) {
                let old_t = new_pdf.remove(&q).unwrap();
                new_pdf.insert(q, old_t + self.pdf_ref(&p));
            } else {
                new_pdf.insert(q, self.pdf_ref(&p));
            }
        }
        // .unwrap() is fine here, because if self is a valid RV, then this also will be.
        MapRandVar::from_map(new_pdf).unwrap()
    }

    pub fn consolidate<Q, RV>(&self, outcomes: BTreeMap<P, RV>) -> Result<MapRandVar<Q, T>, RVError>
    where
        Q: Ord + Seq + Clone + Display,
        RV: RandVar<Q, T>,
    {
        let mut new_pdf: BTreeMap<Q, T> = BTreeMap::new();
        let lb = outcomes.values().map(|rv| rv.lower_bound()).min().unwrap();
        let ub = outcomes.values().map(|rv| rv.upper_bound()).max().unwrap();
        for q in Seq::gen_seq(&lb, &ub) {
            let mut pdf_q = T::zero();
            for p in self.valid_p() {
                if let Some(rv) = outcomes.get(&p) {
                    pdf_q = pdf_q + self.pdf_ref(&p) * rv.pdf_ref(&q);
                } else {
                    return Err(RVError::Other(String::from("every valid p must have an outcome!")));
                }
            }
            if pdf_q > T::zero() {
                new_pdf.insert(q, pdf_q);
            }
        }
        MapRandVar::from_map(new_pdf)
    }
}

impl<P, T> RandVar<P, T> for MapRandVar<P, T>
where
    P: Ord + Clone + Display + Seq,
    T: Num + Sum + Clone + Display + Debug + PartialOrd<T>,
{
    fn build<F: Fn(P) -> T>(lb: P, ub: P, f: F) -> Result<Self, RVError> {
        if lb > ub {
            return Err(RVError::InvalidBounds);
        }
        let mut pdf_map = BTreeMap::new();
        let mut total = T::zero();
        for p in Seq::gen_seq(&lb, &ub) {
            let f_p = f(p.clone());
            if f_p < T::zero() {
                return Err(RVError::NegProb);
            }
            if f_p > T::zero() {
                total = total + f_p.clone();
                pdf_map.insert(p, f_p);
            }
        }
        if T::one() != total {
            return Err(RVError::CDFNotOne);
        }

        Ok(MapRandVar {
            lower_bound: lb,
            upper_bound: ub,
            pdf_map
        })
    }

    fn lower_bound(&self) -> P {
        self.lower_bound.clone()
    }

    fn upper_bound(&self) -> P {
        self.upper_bound.clone()
    }

    unsafe fn raw_pdf(&self, p: &P) -> T {
        if let Some(t) = self.pdf_map.get(p) {
            t.clone()
        } else {
            T::zero()
        }
    }

    fn valid_p(&self) -> SeqGen<P> {
        let items: BTreeSet<P> = self.pdf_map.keys().cloned().collect();
        SeqGen { items }
    }
}

impl<T> NumRandVar<isize, T> for MapRandVar<isize, T>
where
    T: Num + Sum + Clone + Display + Debug + PartialOrd<T> + FromPrimitive
{
    fn convert(&self, p: isize) -> T {
        T::from_isize(p).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use std::cmp;
    use num::{BigInt, BigRational, Rational64, Zero};
    use super::*;

    #[test]
    fn test_consolidate() {
        let d20: RandomVariable<Rational64> = RandomVariable::new_dice(20).unwrap();
        let hit_bonus: RandomVariable<Rational64> = RandomVariable::new_constant(8).unwrap();
        let attack_check = d20
            .add_rv(&hit_bonus)
            .to_map_rv()
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
        let dmg_bonus: RandomVariable<Rational64> = RandomVariable::new_constant(5).unwrap();
        let dmg_dice: RandomVariable<Rational64> = RandomVariable::new_dice(6).unwrap().multiple(2);

        let hit_dmg = dmg_dice.add_rv(&dmg_bonus);
        let crit_dmg = dmg_dice.multiple(2).add_rv(&dmg_bonus);
        let miss_dmg: RandomVariable<Rational64> = RandomVariable::new_constant(0).unwrap();

        let mut outcomes: BTreeMap<isize, RandomVariable<Rational64>> = BTreeMap::new();
        outcomes.insert(0, miss_dmg);
        outcomes.insert(1, hit_dmg);
        outcomes.insert(2, crit_dmg);

        let attack_dmg = attack_check.consolidate(outcomes).unwrap();
        assert_eq!(0, attack_dmg.lower_bound());
        assert_eq!(29, attack_dmg.upper_bound());
        assert_eq!(Rational64::new(8, 20), attack_dmg.pdf(0));
        assert_eq!(Rational64::new(1, 25920), attack_dmg.pdf(29));
        assert_eq!(Rational64::new(151, 20), attack_dmg.expected_value());
    }

    #[test]
    fn test_mixed_add() {
        let d8: RandomVariable<Rational64> = RandomVariable::new_dice(8).unwrap();
        let const_5: MapRandVar<isize, Rational64> = RandomVariable::new_constant(5).unwrap().to_map_rv();
        let d8_plus_5 = d8.add_rv(&const_5);
        assert_eq!(6, d8_plus_5.lower_bound());
        assert_eq!(13, d8_plus_5.upper_bound());
        assert_eq!(Rational64::new(19,2), d8_plus_5.expected_value());
    }

    #[test]
    fn test_4d6_drop_lowest() {
        let d6: RandomVariable<BigRational> = RandomVariable::new_dice(6).unwrap();
        let ability_score = d6
            .to_map_rv()
            .independent_trials_self()
            .independent_trials_self()
            .map_keys(|rolls| {
                let smallest = cmp::min(
                    cmp::min(rolls.0.0, rolls.0.1),
                    cmp::min(rolls.1.0, rolls.1.1));
                rolls.0.0 + rolls.0.1 + rolls.1.0 + rolls.1.1 - smallest
            }).to_vec_rv();
        assert_eq!(3, ability_score.lower_bound());
        assert_eq!(18, ability_score.upper_bound());
        assert_eq!(BigRational::new(BigInt::from_isize(15869).unwrap(), BigInt::from_isize(1296).unwrap()), ability_score.expected_value());
    }

    #[test]
    fn test_super_adv_two_ways() {
        let d20_super_adv: RandomVariable<Rational64> = RandomVariable::new_dice(20).unwrap().max_three_trials();
        let other_d20: MapRandVar<isize, Rational64> = RandomVariable::new_dice(20).unwrap().to_map_rv();
        let other_d20_super_adv = other_d20
            .independent_trials(&other_d20)
            .independent_trials(&other_d20)
            .map_keys(|pair| cmp::max(cmp::max(pair.0.0, pair.0.1), pair.1))
            .to_vec_rv();
        assert_eq!(d20_super_adv, other_d20_super_adv);
    }

    #[test]
    fn test_adv_two_ways() {
        let d20_adv: RandomVariable<Rational64> = RandomVariable::new_dice(20).unwrap().max_two_trials();
        let other_d20: MapRandVar<isize, Rational64> = RandomVariable::new_dice(20).unwrap().to_map_rv();
        let other_d20_adv = other_d20
            .independent_trials(&other_d20)
            .map_keys(|pair| cmp::max(pair.0, pair.1))
            .to_vec_rv();
        assert_eq!(d20_adv, other_d20_adv);
    }

    #[test]
    fn test_2d6_two_ways() {
        let two_d6: RandomVariable<Rational64> = RandomVariable::new_dice(6).unwrap().multiple(2);
        let other_d6: MapRandVar<isize, Rational64> = RandomVariable::new_dice(6).unwrap().to_map_rv();
        let other_2d6 = other_d6
            .independent_trials(&other_d6)
            .map_keys(|pair| pair.0 + pair.1)
            .to_vec_rv();
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

    #[test]
    fn test_const() {
        let rv: RandomVariable<Rational64> = RandomVariable::new_constant(5).unwrap();
        assert_eq!(5, rv.lower_bound());
        assert_eq!(5, rv.upper_bound());
        assert_eq!(Rational64::zero(), rv.pdf(4));
        assert_eq!(Rational64::one(), rv.pdf(5));
        assert_eq!(Rational64::zero(), rv.pdf(6));
        assert_eq!(Rational64::new(5,1), rv.expected_value());
        assert_eq!(Rational64::zero(), rv.variance());
    }

    #[test]
    fn test_const_neg() {
        let rv: RandomVariable<Rational64> = RandomVariable::new_constant(-7).unwrap();
        assert_eq!(-7, rv.lower_bound());
        assert_eq!(-7, rv.upper_bound());
        assert_eq!(Rational64::zero(), rv.pdf(-8));
        assert_eq!(Rational64::one(), rv.pdf(-7));
        assert_eq!(Rational64::zero(), rv.pdf(-6));
        assert_eq!(Rational64::new(-7,1), rv.expected_value());
        assert_eq!(Rational64::zero(), rv.variance());
    }

    #[test]
    fn test_unif() {
        let rv: RandomVariable<Rational64> = RandomVariable::new_uniform(3,14).unwrap();
        assert_eq!(3, rv.lower_bound());
        assert_eq!(14, rv.upper_bound());
        assert_eq!(Rational64::zero(), rv.pdf(2));
        assert_eq!(Rational64::zero(), rv.pdf(15));
        let mut total = Rational64::zero();
        for x in 3..=14 {
            assert_eq!(Rational64::new(1,12), rv.pdf(x));
            total += rv.pdf(x);
            assert_eq!(total, rv.cdf(x));
        }
        assert_eq!(Rational64::one(), total);
        assert_eq!(Rational64::new(17,2), rv.expected_value());
        assert_eq!(Rational64::new(143,12), rv.variance());
    }

    #[test]
    fn test_d6() {
        let rv: RandomVariable<Rational64> = RandomVariable::new_dice(6).unwrap();
        assert_eq!(1, rv.lower_bound());
        assert_eq!(6, rv.upper_bound());
        assert_eq!(Rational64::zero(), rv.pdf(0));
        assert_eq!(Rational64::zero(), rv.pdf(7));
        let mut total = Rational64::zero();
        for x in 1..=6 {
            assert_eq!(Rational64::new(1,6), rv.pdf(x));
            total += rv.pdf(x);
            assert_eq!(total, rv.cdf(x));
        }
        assert_eq!(Rational64::one(), total);
        assert_eq!(Rational64::new(7,2), rv.expected_value());
        assert_eq!(Rational64::new(35,12), rv.variance());
    }

    #[test]
    fn test_d10r2() {
        let rv: RandomVariable<Rational64> = RandomVariable::new_dice_reroll(10, 2).unwrap();
        assert_eq!(1, rv.lower_bound());
        assert_eq!(10, rv.upper_bound());
        assert_eq!(Rational64::zero(), rv.pdf(0));
        assert_eq!(Rational64::zero(), rv.pdf(11));
        let mut total = Rational64::zero();
        for x in 1..=10 {
            if x < 3 {
                assert_eq!(Rational64::new(2,100), rv.pdf(x));
            } else {
                assert_eq!(Rational64::new(12,100), rv.pdf(x));
            }
            total += rv.pdf(x);
            assert_eq!(total, rv.cdf(x));
        }
        assert_eq!(Rational64::one(), total);
        assert_eq!(Rational64::new(63,10), rv.expected_value());
        assert_eq!(Rational64::new(601,100), rv.variance());
    }
}
