use std::{cmp, fmt};
use std::collections::BTreeSet;
use std::fmt::{Display, Formatter};
use std::iter::Sum;
use num::{Num, PrimInt};

#[derive(Clone)]
pub struct SeqGen<T: Ord + Clone> {
    pub items: BTreeSet<T>,
}

impl<T: Ord + Clone> Iterator for SeqGen<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.items.pop_first()
    }
}

pub trait Seq
where
    Self: Ord + Clone + Sized
{
    fn gen_seq(&self, other: &Self) -> SeqGen<Self>;
}

impl Seq for isize {
    fn gen_seq(&self, other: &Self) -> SeqGen<Self> {
        let first = *cmp::min(self, other);
        let second = *cmp::max(self, other);
        let items = BTreeSet::from_iter(first..=second);
        SeqGen { items }
    }
}

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Clone)]
pub struct Pair<A: Ord + Clone, B: Ord + Clone>(pub A, pub B);

impl<A, B> Seq for Pair<A, B>
where
    A: Seq + Clone,
    B: Seq + Clone,
{
    fn gen_seq(&self, other: &Self) -> SeqGen<Self> {
        let a_sg = Seq::gen_seq(&self.0, &other.0);
        let b_sg = Seq::gen_seq(&self.1, &other.1);
        let ab_set: BTreeSet<Pair<A, B>> = itertools::iproduct!(a_sg, b_sg).map(|(a, b)| Pair(a, b)).collect();
        SeqGen { items: ab_set }
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

#[derive(Debug)]
pub enum RVError {
    InvalidBounds,
    CDFNotOne,
    NegProb,
    NoRound,
    Other(String),
}

pub trait RandVar<P, T>
    where
        P: Ord + Seq + Display,
        T: Num + Sum + Display,
{
    fn build<F: Fn(P) -> T>(lb: P, ub: P, f: F) -> Result<Self, RVError> where Self: Sized;
    fn lower_bound(&self) -> P;
    fn upper_bound(&self) -> P;
    unsafe fn raw_pdf(&self, p: &P) -> T;
    fn valid_p(&self) -> SeqGen<P>;

    fn pdf_ref(&self, p: &P) -> T {
        if (&self.lower_bound() <= p) && (p <= &self.upper_bound()) {
            unsafe {
                self.raw_pdf(p)
            }
        } else {
            T::zero()
        }
    }
    fn pdf(&self, p: P) -> T {
        self.pdf_ref(&p)
    }

    fn cdf_ref(&self, p: &P) -> T {
        if &self.upper_bound() <= p {
            T::one()
        } else if &self.lower_bound() <= p {
            self.valid_p().take_while(|x| x <= p).map(|x| self.pdf(x)).sum()
        } else {
            T::zero()
        }
    }
    fn cdf(&self, p: P) -> T {
        self.cdf_ref(&p)
    }

    fn cdf_exclusive_ref(&self, p: &P) -> T {
        if &self.upper_bound() < p {
            T::one()
        } else if &self.lower_bound() < p {
            self.valid_p().take_while(|x| x < p).map(|x| self.pdf(x)).sum()
        } else {
            T::zero()
        }
    }
    fn cdf_exclusive(&self, p: P) -> T {
        self.cdf_exclusive_ref(&p)
    }

    fn cap_lb(&self, lb: P) -> Result<Self, RVError>
        where
            P: Copy,
            Self: Sized
    {
        RandVar::build(lb, self.upper_bound(), |p| {
            if p == lb {
                self.cdf(lb)
            } else {
                self.pdf(p)
            }
        })
    }

    fn cap_ub(&self, ub: P) -> Result<Self, RVError>
        where
            P: Copy,
            Self: Sized
    {
        RandVar::build(self.lower_bound(), ub, |p| {
            if p < ub {
                self.pdf(p)
            } else { // p == ub
                T::one() - self.cdf_exclusive(ub)
            }
        })
    }

    fn reroll_once_on<F>(&self, pred: F) -> Self
    where
        F: Fn(&P) -> bool,
        Self: Sized,
        T: Clone,
    {
        let mut reroll_chance = T::zero();
        for p in self.valid_p() {
            if pred(&p) {
                reroll_chance = reroll_chance + self.pdf(p);
            }
        }

        let reroll_pdf = |p| {
            if pred(&p) {
                reroll_chance.clone() * self.pdf(p)
            } else {
                (T::one() + reroll_chance.clone()) * self.pdf(p)
            }
        };
        // .unwrap() is fine here, because if self is a valid RV, then this also will be.
        RandVar::build(self.lower_bound(), self.upper_bound(), reroll_pdf).unwrap()
    }

    fn max_two_trials(&self) -> Self
        where
            T: Clone,
            Self: Sized
    {
        let max_pdf = |p| {
            (T::one()+T::one()) * self.pdf_ref(&p) * self.cdf_exclusive_ref(&p) + num::pow(self.pdf_ref(&p), 2)
        };
        // .unwrap() is fine here, because if self is a valid RV, then this also will be.
        RandVar::build(self.lower_bound(), self.upper_bound(), max_pdf).unwrap()
    }

    fn min_two_trials(&self) -> Self
        where
            T: Clone,
            Self: Sized
    {
        let min_pdf = |p| {
            let max_pdf = (T::one()+T::one()) * self.pdf_ref(&p) * self.cdf_exclusive_ref(&p) + num::pow(self.pdf_ref(&p), 2);
            (T::one()+T::one()) * self.pdf_ref(&p) - max_pdf
        };
        // .unwrap() is fine here, because if self is a valid RV, then this also will be.
        RandVar::build(self.lower_bound(), self.upper_bound(), min_pdf).unwrap()
    }

    fn max_three_trials(&self) -> Self
        where
            T: Clone,
            Self: Sized
    {
        let max_pdf = |p| {
            let x = (T::one() + T::one() + T::one()) * self.pdf_ref(&p) * num::pow(self.cdf_exclusive_ref(&p), 2);
            let y = (T::one() + T::one() + T::one()) * num::pow(self.pdf_ref(&p), 2) * self.cdf_exclusive_ref(&p);
            x + y + num::pow(self.pdf_ref(&p), 3)
        };
        // .unwrap() is fine here, because if self is a valid RV, then this also will be.
        RandVar::build(self.lower_bound(), self.upper_bound(), max_pdf).unwrap()
    }

    fn print_distributions(&self) {
        println!("P:\tpdf\t(cdf)");
        for p in self.valid_p() {
            println!("{}:\t{}\t({})", &p, self.pdf_ref(&p), self.cdf_ref(&p));
        }
    }

    fn print_pdf<F, T2>(&self, f: F)
        where
            F: Fn(T) -> T2,
            T2: Display,
    {
        println!("P\tpdf\t~pdf");
        for p in self.valid_p() {
            println!("{}:\t{}\t~{}", &p, self.pdf_ref(&p), f(self.pdf_ref(&p)));
        }
    }
}

fn convolution<P, T, F1, F2>(lb: P, ub: P, f1: F1, f2: F2, x: P) -> T
    where
        P: PrimInt,
        T: Num + Sum,
        F1: Fn(P) -> T,
        F2: Fn(P) -> T,
{
    num::range_inclusive(lb, ub).map(|y| f1(x-y)*f2(y)).sum()
}

pub trait NumRandVar<P, T>: RandVar<P, T>
    where
        P: PrimInt + Seq + Display,
        T: Num + Sum + Clone + Display,
{
    fn convert(&self, p: P) -> T;

    fn expected_value(&self) -> T {
        self.general_expected_value(|p| self.convert(p))
    }

    fn general_expected_value<F>(&self, f: F) -> T
        where
            F: Fn(P) -> T
    {
        let mut result = T::zero();
        for p in self.valid_p() {
            result = result + f(p) * self.pdf(p);
        }
        result
    }

    fn variance(&self) -> T {
        let ev = self.expected_value();
        let sq_ev = self.general_expected_value(|p| num::pow(self.convert(p),2));
        sq_ev - num::pow(ev, 2)
    }

    fn print_stats(&self) {
        println!("ev  = {}", self.expected_value());
        println!("var = {}", self.variance());
    }

    fn print_stats_convert<F,T2>(&self, f: F)
        where
            F: Fn(T) -> T2,
            T2: Display
    {
        println!("ev  = {} ~= {}", self.expected_value(), f(self.expected_value()));
        println!("var = {} ~= {}", self.variance(), f(self.variance()));
    }

    fn add_const(&self, p: P) -> Self
    where
        Self: Sized
    {
        // .unwrap() is fine here, because if self is a valid RV, then this also will be.
        RandVar::build(
            self.lower_bound() + p,
            self.upper_bound() + p,
            |x| self.pdf(x-p)
        ).unwrap()
    }

    fn add_rv(&self, other: &impl NumRandVar<P,T>) -> Self
    where
        Self: Sized
    {
        let new_lb = self.lower_bound() + other.lower_bound();
        let new_ub = self.upper_bound() + other.upper_bound();
        let min_lb = cmp::min(self.lower_bound(), other.lower_bound());
        let max_ub = cmp::max(self.upper_bound(), other.upper_bound());
        // .unwrap() is fine here, because if self and other are valid RVs, then this also will be.
        RandVar::build(
            new_lb,
            new_ub,
            |x| convolution(
                min_lb,
                max_ub,
                |p1| self.pdf(p1),
                |p2| other.pdf(p2),
                x)).unwrap()
    }

    fn multiple(&self, num_times: i32) -> Self
        where
            Self: Sized + Clone
    {
        if num_times == 0 {
            return RandVar::build(P::zero(), P::zero(), |_| T::one()).unwrap();
        } else if num_times == 1 {
            return self.clone();
        } else if num_times == -1 {
            return self.opposite_rv();
        }
        let is_neg = num_times < 0;
        let pos_num = num::abs(num_times);
        let mut rv = self.add_rv(self);
        for _ in 2..pos_num {
            rv = rv.add_rv(self);
        }
        if is_neg {
            rv.opposite_rv()
        } else {
            rv
        }
    }

    fn minus_rv(&self, other: &impl NumRandVar<P,T>) -> Self
        where
            Self: Sized
    {
        self.add_rv(&other.opposite_rv())
    }

    fn opposite_rv(&self) -> Self
        where
            Self: Sized
    {
        // .unwrap() is fine here, because if self is a valid RV, then this also will be.
        RandVar::build(
            P::zero()-self.upper_bound(),
            P::zero()-self.lower_bound(),
            |p| self.pdf(P::zero()-p)).unwrap()
    }

    // This only works on P's that are integer-like. Floats or rationals won't work.
    fn half(&self) -> Result<Self, RVError>
        where
            Self: Sized
    {
        let two = P::one() + P::one();
        // P requires PrimInt for this trait, so this should never
        // happen, but check for rounding anyway.
        if P::one()/two != P::zero() {
            return Err(RVError::NoRound);
        }
        let lb = self.lower_bound()/two;
        let ub = self.upper_bound()/two;
        RandVar::build(lb, ub, |p| {
            if p > P::zero() {
                self.pdf(two*p) + self.pdf(two*p + P::one())
            } else if p == P::zero() {
                self.pdf(P::zero()-P::one()) + self.pdf(P::zero()) + self.pdf(P::one())
            } else {
                self.pdf(two*p) + self.pdf(two*p - P::one())
            }
        })
    }

    fn prob_lt(&self, other: &impl NumRandVar<P,T>) -> T
        where
            Self: Sized
    {
        self.minus_rv(other).cdf_exclusive(P::zero())
    }

    fn prob_le(&self, other: &impl NumRandVar<P,T>) -> T
        where
            Self: Sized
    {
        self.minus_rv(other).cdf(P::zero())
    }

    fn prob_eq(&self, other: &impl NumRandVar<P,T>) -> T
        where
            Self: Sized
    {
        self.minus_rv(other).pdf(P::zero())
    }

    fn prob_gt(&self, other: &impl NumRandVar<P,T>) -> T
        where
            Self: Sized
    {
        let diff_rv = self.minus_rv(other);
        T::one() - diff_rv.cdf(P::zero())
    }

    fn prob_ge(&self, other: &impl NumRandVar<P,T>) -> T
        where
            Self: Sized
    {
        let diff_rv = self.minus_rv(other);
        T::one() - diff_rv.cdf_exclusive(P::zero())
    }
}


#[cfg(test)]
mod tests {
    use num::{BigInt, BigRational, One, Rational64, FromPrimitive, Zero};
    use crate::RandomVariable;
    use super::*;

    #[test]
    fn test_reroll() {
        let rv1: RandomVariable<Rational64> = RandomVariable::new_dice_reroll(10,2).unwrap();
        let rv2: RandomVariable<Rational64> = RandomVariable::new_dice(10).unwrap().reroll_once_on(|p| *p <= 2);
        assert_eq!(rv1, rv2);

        let rv3: RandomVariable<Rational64> = RandomVariable::new_dice_reroll(20,1).unwrap();
        let rv4: RandomVariable<Rational64> = RandomVariable::new_dice(20).unwrap().reroll_once_on(|p| *p <= 1);
        assert_eq!(rv3, rv4);
    }

    #[test]
    fn test_minus_d4() {
        let rv: RandomVariable<Rational64> = RandomVariable::new_dice(4).unwrap().opposite_rv();
        assert_eq!(-4, rv.lower_bound());
        assert_eq!(-1, rv.upper_bound());
        assert_eq!(Rational64::zero(), rv.pdf(0));
        assert_eq!(Rational64::zero(), rv.pdf(-5));
        let mut total = Rational64::zero();
        for x in -4..=-1 {
            assert_eq!(Rational64::new(1,4), rv.pdf(x));
            total += rv.pdf(x);
            assert_eq!(total, rv.cdf(x));
        }
        assert_eq!(Rational64::one(), total);
        assert_eq!(Rational64::new(-5,2), rv.expected_value());
        assert_eq!(Rational64::new(5,4), rv.variance());
    }

    #[test]
    fn test_d20_minus_5_cap_lb() {
        let rv1: RandomVariable<Rational64> = RandomVariable::new_dice(20).unwrap();
        let rv2: RandomVariable<Rational64> = RandomVariable::new_constant(5).unwrap();
        let rv = rv1.minus_rv(&rv2).cap_lb(0).unwrap();
        assert_eq!(0, rv.lower_bound());
        assert_eq!(15, rv.upper_bound());
        assert_eq!(Rational64::zero(), rv.pdf(-1));
        assert_eq!(Rational64::zero(), rv.pdf(16));
        let mut total = Rational64::zero();
        for x in 0..=15 {
            if x == 0 {
                assert_eq!(Rational64::new(5,20), rv.pdf(x));
            } else {
                assert_eq!(Rational64::new(1,20), rv.pdf(x));
            }
            total += rv.pdf(x);
            assert_eq!(total, rv.cdf(x));
        }
        assert_eq!(Rational64::one(), total);
        assert_eq!(Rational64::new(6,1), rv.expected_value());
        assert_eq!(Rational64::new(26,1), rv.variance());
    }

    #[test]
    fn test_d11_plus_3_cap_ub() {
        let rv1: RandomVariable<Rational64> = RandomVariable::new_dice(11).unwrap();
        let rv2: RandomVariable<Rational64> = RandomVariable::new_constant(3).unwrap();
        let rv = rv1.add_rv(&rv2).cap_ub(10).unwrap();
        assert_eq!(4, rv.lower_bound());
        assert_eq!(10, rv.upper_bound());
        assert_eq!(Rational64::zero(), rv.pdf(3));
        assert_eq!(Rational64::zero(), rv.pdf(11));
        let mut total = Rational64::zero();
        for x in 4..=10 {
            if x == 10 {
                assert_eq!(Rational64::new(5,11), rv.pdf(x));
            } else {
                assert_eq!(Rational64::new(1,11), rv.pdf(x));
            }
            total += rv.pdf(x);
            assert_eq!(total, rv.cdf(x));
        }
        assert_eq!(Rational64::one(), total);
        assert_eq!(Rational64::new(89,11), rv.expected_value());
        assert_eq!(Rational64::new(560,121), rv.variance());

        let rv3 = rv1.add_const(3).cap_ub(10).unwrap();
        assert_eq!(rv, rv3);
    }

    #[test]
    fn test_d12_minus_d8() {
        let rv1: RandomVariable<Rational64> = RandomVariable::new_dice(12).unwrap();
        let rv2: RandomVariable<Rational64> = RandomVariable::new_dice(8).unwrap();
        let rv = rv1.minus_rv(&rv2);
        assert_eq!(-7, rv.lower_bound());
        assert_eq!(11, rv.upper_bound());
        assert_eq!(Rational64::zero(), rv.pdf(-8));
        assert_eq!(Rational64::zero(), rv.pdf(12));
        let mut total = Rational64::zero();
        for x in -7..=11 {
            total += rv.pdf(x);
            assert_eq!(total, rv.cdf(x));
        }
        assert_eq!(Rational64::one(), total);
        assert_eq!(Rational64::new(2,1), rv.expected_value());
        assert_eq!(Rational64::new(103,6), rv.variance());
    }

    #[test]
    fn test_2d6() {
        let rv1: RandomVariable<BigRational> = RandomVariable::new_dice(6).unwrap();
        let rv2: RandomVariable<BigRational> = RandomVariable::new_dice(6).unwrap();
        let rv = rv1.add_rv(&rv2);
        assert_eq!(2, rv.lower_bound());
        assert_eq!(12, rv.upper_bound());
        assert_eq!(BigRational::zero(), rv.pdf(1));
        assert_eq!(BigRational::zero(), rv.pdf(13));
        let mut total = BigRational::zero();
        for x in 2..=12 {
            let numerator = 6 - num::abs(7-x);
            let pdf_x = BigRational::new(BigInt::from_isize(numerator).unwrap(), BigInt::from_isize(36).unwrap());
            assert_eq!(pdf_x, rv.pdf(x));
            total += rv.pdf(x);
            assert_eq!(total, rv.cdf(x));
        }
        assert_eq!(BigRational::one(), total);
        assert_eq!(BigRational::from_i32(7).unwrap(), rv.expected_value());
    }

    #[test]
    fn test_multiple() {
        let rv1: RandomVariable<BigRational> = RandomVariable::new_dice(6).unwrap();
        let rv = rv1.add_rv(&rv1);
        let other_rv = rv1.multiple(2);
        assert_eq!(rv, other_rv);

        let rv = rv1.add_rv(&rv1).add_rv(&rv1);
        let other_rv = rv1.multiple(3);
        assert_eq!(rv, other_rv);
    }

    #[test]
    fn test_fireball() {
        let d6: RandomVariable<BigRational> = RandomVariable::new_dice(6).unwrap();
        let fireball = d6.multiple(8);
        assert_eq!(8, fireball.lower_bound());
        assert_eq!(48, fireball.upper_bound());
        assert_eq!(BigRational::from_i32(28).unwrap(), fireball.expected_value());

        let fireball_resist = fireball.half().unwrap();
        assert_eq!(4, fireball_resist.lower_bound());
        assert_eq!(24, fireball_resist.upper_bound());
        let mut total = BigRational::zero();
        for x in 4..=24 {
            assert_eq!(fireball.pdf(2*x) + fireball.pdf(2*x+1), fireball_resist.pdf(x));
            total += fireball_resist.pdf(x);
            assert_eq!(total, fireball_resist.cdf(x));
        }
        assert_eq!(BigRational::one(), total);
        let ev = BigRational::new(BigInt::from_i32(55).unwrap(), BigInt::from_i32(4).unwrap());
        assert_eq!(ev, fireball_resist.expected_value());
    }

    #[test]
    fn test_half() {
        let rv: RandomVariable<Rational64> = RandomVariable::new_uniform(-3,7).unwrap().half().unwrap();
        assert_eq!(-1, rv.lower_bound());
        assert_eq!(3, rv.upper_bound());

        let mut total = Rational64::zero();
        for x in -1..=3 {
            if x == 0 {
                assert_eq!(Rational64::new(3, 11), rv.pdf(x));
            } else {
                assert_eq!(Rational64::new(2, 11), rv.pdf(x));
            }
            total += rv.pdf(x);
            assert_eq!(total, rv.cdf(x));
        }
        assert_eq!(Rational64::one(), total);
    }

    #[test]
    fn test_d20_adv() {
        let rv: RandomVariable<Rational64> = RandomVariable::new_dice(20).unwrap().max_two_trials();
        assert_eq!(1, rv.lower_bound());
        assert_eq!(20, rv.upper_bound());
        assert_eq!(Rational64::zero(), rv.pdf(0));
        assert_eq!(Rational64::zero(), rv.pdf(21));

        let mut total = Rational64::zero();
        for x in 1..=20 {
            assert_eq!(Rational64::new(2*x-1,400), rv.pdf(x as isize));
            total += rv.pdf(x as isize);
            assert_eq!(total, rv.cdf(x as isize));
        }
        assert_eq!(Rational64::one(), total);
        assert_eq!(Rational64::new(553,40), rv.expected_value());
    }

    #[test]
    fn test_d20_disadv() {
        let rv: RandomVariable<Rational64> = RandomVariable::new_dice(20).unwrap().min_two_trials();
        assert_eq!(1, rv.lower_bound());
        assert_eq!(20, rv.upper_bound());
        assert_eq!(Rational64::zero(), rv.pdf(0));
        assert_eq!(Rational64::zero(), rv.pdf(21));

        let mut total = Rational64::zero();
        for x in 1..=20 {
            assert_eq!(Rational64::new(2*(21-x)-1,400), rv.pdf(x as isize));
            total += rv.pdf(x as isize);
            assert_eq!(total, rv.cdf(x as isize));
        }
        assert_eq!(Rational64::one(), total);
        assert_eq!(Rational64::new(287,40), rv.expected_value());
    }

    #[test]
    fn test_d20_super_adv() {
        let rv: RandomVariable<Rational64> = RandomVariable::new_dice(20).unwrap().max_three_trials();
        assert_eq!(1, rv.lower_bound());
        assert_eq!(20, rv.upper_bound());
        assert_eq!(Rational64::zero(), rv.pdf(0));
        assert_eq!(Rational64::zero(), rv.pdf(21));
        let mut total = Rational64::zero();
        for x in 1..=20 {
            assert_eq!(Rational64::new(3*x*x-3*x+1,8000), rv.pdf(x as isize));
            total += rv.pdf(x as isize);
            assert_eq!(total, rv.cdf(x as isize));
        }
        assert_eq!(Rational64::one(), total);
        assert_eq!(Rational64::new(1239,80), rv.expected_value());
    }

    #[test]
    fn test_cmp_rv() {
        let d6: RandomVariable<Rational64> = RandomVariable::new_dice(6).unwrap();
        assert_eq!(Rational64::new(5,12), d6.prob_gt(&d6));
        assert_eq!(Rational64::new(7,12), d6.prob_ge(&d6));
        assert_eq!(Rational64::new(1,6), d6.prob_eq(&d6));
        assert_eq!(Rational64::new(7,12), d6.prob_le(&d6));
        assert_eq!(Rational64::new(5,12), d6.prob_gt(&d6));

        let d20: RandomVariable<Rational64> = RandomVariable::new_dice(20).unwrap();
        let d20_adv = d20.max_two_trials();
        assert_eq!(Rational64::one(), d20_adv.prob_ge(&d20) + d20_adv.prob_lt(&d20));
        assert_eq!(Rational64::one(), d20_adv.prob_gt(&d20) + d20_adv.prob_lt(&d20) + d20_adv.prob_eq(&d20));
    }
}
