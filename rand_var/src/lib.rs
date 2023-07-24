use std::cmp;
use std::collections::BTreeMap;
use std::fmt::{Debug, Display};
use std::iter::Sum;
use num::{FromPrimitive, Integer, Num, One};
use num::rational::Ratio;
use crate::rv_traits::{NumRandVar, RandVar, SeqGen};

pub mod rv_traits;

#[derive(PartialEq, Clone, Debug)]
pub struct RandomVariable<T: Num> {
    lower_bound: isize,
    upper_bound: isize,
    pdf_vec: Vec<T>,
}

impl<T> RandomVariable<Ratio<T>>
where
    T: Integer + Debug + Clone + FromPrimitive + Display
{
    pub fn new_dice(sides: isize) -> Self {
        assert!(sides > 0, "sides must be positive");
        <RandomVariable<Ratio<T>> as RandVar<isize, Ratio<T>>>::build(
            1,
            sides,
            |_| {
                Ratio::new(T::one(), T::from_isize(sides).unwrap())
            })
    }

    pub fn new_dice_reroll(sides: isize, reroll_max: isize) -> Self {
        assert!(sides > 0, "sides must be positive");
        assert!(reroll_max > 0, "reroll_max must be positive");
        assert!(sides > reroll_max, "sides must be larger than reroll_max");
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

    pub fn new_constant(value: isize) -> Self {
        <RandomVariable<Ratio<T>> as RandVar<isize, Ratio<T>>>::build(
            value,
            value,
            |_| Ratio::one())
    }

    pub fn new_uniform(lb: isize, ub: isize) -> Self {
        <RandomVariable<Ratio<T>> as RandVar<isize, Ratio<T>>>::build(
            lb,
            ub,
            |_| {
                Ratio::new(T::one(), T::from_isize(ub-lb+1).unwrap())
            })
    }
}

impl SeqGen for isize {
    fn gen_seq_p(&self, other: &Self) -> Box<dyn Iterator<Item=Self>> {
        let first = *cmp::min(self, other);
        let second = *cmp::max(self, other);
        Box::new(first..=second)
    }
}

impl<T> RandVar<isize, T> for RandomVariable<T>
where
    T: Num + Sum + Debug + Clone + Display
{
    fn build<F>(lb: isize, ub: isize, f: F) -> RandomVariable<T>
    where
        F: Fn(isize) -> T
    {
        assert!(lb <= ub, "lower bound must be <= upper bound");
        let length: usize = (ub - lb + 1) as usize;
        let mut pdf_vec = Vec::with_capacity(length);
        let mut total = T::zero();
        for i in SeqGen::gen_seq_p(&lb, &ub) {
            total = total + f(i);
            pdf_vec.push(f(i));
        }
        assert_eq!(T::one(), total, "cdf(upper bound) must be 1");

        RandomVariable {
            lower_bound: lb,
            upper_bound: ub,
            pdf_vec
        }
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

    fn valid_p(&self) -> Box<dyn Iterator<Item=isize> + '_> {
        <isize as SeqGen>::gen_seq_p(&self.lower_bound(), &self.upper_bound())
    }
}

impl<T> NumRandVar<isize, T> for RandomVariable<T>
where
    T: Num + Sum + Debug + Clone + FromPrimitive + Display
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


impl<P, T> RandVar<P, T> for MapRandVar<P, T>
where
    P: Ord + Clone + Display + SeqGen,
    T: Num + Sum + Clone + Display + Debug,
{
    fn build<F: Fn(P) -> T>(lb: P, ub: P, f: F) -> Self {
        assert!(lb <= ub, "lower bound must be <= upper bound");
        let mut pdf_map = BTreeMap::new();
        let mut total = T::zero();
        for p in SeqGen::gen_seq_p(&lb, &ub) {
            let f_p = f(p.clone());
            total = total + f_p.clone();
            pdf_map.insert(p, f_p);
        }
        assert_eq!(T::one(), total, "cdf(upper bound) must be 1");

        MapRandVar {
            lower_bound: lb,
            upper_bound: ub,
            pdf_map
        }
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

    fn valid_p(&self) -> Box<dyn Iterator<Item=P> + '_> {
        Box::new(self.pdf_map.keys().cloned())
    }
}


#[cfg(test)]
mod tests {
    use num::{Rational64, Zero};
    use super::*;

    #[test]
    fn test_const() {
        let rv: RandomVariable<Rational64> = RandomVariable::new_constant(5);
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
        let rv: RandomVariable<Rational64> = RandomVariable::new_constant(-7);
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
        let rv: RandomVariable<Rational64> = RandomVariable::new_uniform(3,14);
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
        let rv: RandomVariable<Rational64> = RandomVariable::new_dice(6);
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
        let rv: RandomVariable<Rational64> = RandomVariable::new_dice_reroll(10, 2);
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