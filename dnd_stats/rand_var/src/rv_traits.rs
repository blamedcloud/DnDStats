use std::cmp;
use std::collections::BTreeSet;
use std::fmt::Display;
use std::iter::Sum;
use std::ops::{Add, Div, Mul, Sub};
use num::{One, Zero};
use crate::rv_traits::sequential::{Seq, SeqIter};

pub mod sequential;

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
        P: Ord + Clone,
        T: Zero + One + Sum + Add<T, Output=T> + Sub<T, Output=T> + Mul<T, Output=T> + Clone,
{
    fn build<F: Fn(P) -> T>(seq_iter: SeqIter<P>, f: F) -> Result<Self, RVError> where Self: Sized;
    fn lower_bound(&self) -> P;
    fn upper_bound(&self) -> P;
    unsafe fn raw_pdf(&self, p: &P) -> T;
    fn valid_p(&self) -> SeqIter<P>;

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

    fn general_expected_value<F>(&self, f: F) -> T
    where
        F: Fn(P) -> T
    {
        let mut result = T::zero();
        for p in self.valid_p() {
            result = result + self.pdf_ref(&p) * f(p);
        }
        result
    }

    fn cap_lb(&self, lb: P) -> Result<Self, RVError>
    where
        Self: Sized + Clone,
    {
        if lb > self.lower_bound() {
            let seq_iter = SeqIter { items: self.valid_p().filter(|p| *p >= lb).collect() };
            RandVar::build(seq_iter, |p| {
                if p == lb {
                    self.cdf_ref(&lb)
                } else {
                    self.pdf(p)
                }
            })
        } else {
            Ok(self.clone())
        }
    }

    fn cap_ub(&self, ub: P) -> Result<Self, RVError>
    where
        Self: Sized + Clone,
    {
        if ub < self.upper_bound() {
            let seq_iter = SeqIter { items: self.valid_p().filter(|p| *p <= ub).collect() };
            RandVar::build(seq_iter, |p| {
                if p < ub {
                    self.pdf(p)
                } else { // p == ub
                    T::one() - self.cdf_exclusive_ref(&ub)
                }
            })
        } else {
            Ok(self.clone())
        }
    }

    fn reroll_once_on<F>(&self, pred: F) -> Self
    where
        F: Fn(&P) -> bool,
        Self: Sized,
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
        RandVar::build(self.valid_p(), reroll_pdf).unwrap()
    }

    fn max_two_trials(&self) -> Self
    where
        Self: Sized
    {
        let max_pdf = |p| {
            (T::one()+T::one()) * self.pdf_ref(&p) * self.cdf_exclusive_ref(&p) + num::pow(self.pdf_ref(&p), 2)
        };
        // .unwrap() is fine here, because if self is a valid RV, then this also will be.
        RandVar::build(self.valid_p(), max_pdf).unwrap()
    }

    fn min_two_trials(&self) -> Self
    where
        Self: Sized
    {
        let min_pdf = |p| {
            let max_pdf = (T::one()+T::one()) * self.pdf_ref(&p) * self.cdf_exclusive_ref(&p) + num::pow(self.pdf_ref(&p), 2);
            (T::one()+T::one()) * self.pdf_ref(&p) - max_pdf
        };
        // .unwrap() is fine here, because if self is a valid RV, then this also will be.
        RandVar::build(self.valid_p(), min_pdf).unwrap()
    }

    fn max_three_trials(&self) -> Self
    where
        Self: Sized
    {
        let max_pdf = |p| {
            let x = (T::one() + T::one() + T::one()) * self.pdf_ref(&p) * num::pow(self.cdf_exclusive_ref(&p), 2);
            let y = (T::one() + T::one() + T::one()) * num::pow(self.pdf_ref(&p), 2) * self.cdf_exclusive_ref(&p);
            x + y + num::pow(self.pdf_ref(&p), 3)
        };
        // .unwrap() is fine here, because if self is a valid RV, then this also will be.
        RandVar::build(self.valid_p(), max_pdf).unwrap()
    }

    fn print_distributions(&self)
    where
        P: Display,
        T: Display,
    {
        println!("P:\tpdf\t(cdf)");
        for p in self.valid_p() {
            println!("{}:\t{}\t({})", &p, self.pdf_ref(&p), self.cdf_ref(&p));
        }
    }

    fn print_pdf<F, T2>(&self, f: F)
    where
        P: Display,
        T: Display,
        F: Fn(T) -> T2,
        T2: Display,
    {
        println!("P\tpdf\t~pdf");
        for p in self.valid_p() {
            println!("{}:\t{}\t~{}", &p, self.pdf_ref(&p), f(self.pdf_ref(&p)));
        }
    }
}

// any sane numeric type ought to have 1/10 + 2/10 == 3/10
fn sanity_check<P>() -> bool
where
    P: One + Add<P, Output=P> + Div<P, Output=P> + Clone + PartialEq
{
    let two = P::one() + P::one();
    let three = P::one() + P::one() + P::one();
    let ten = three.clone() + three.clone() + three.clone() + P::one();
    let left = (P::one()/ten.clone()) + (two/ten.clone());
    let right = three/ten;
    left == right
}

fn convolution<P, T, F1, F2>(lb: P, ub: P, f1: F1, f2: F2, x: P) -> T
where
    P: Seq + Sub<P,Output=P>,
    T: Mul<T, Output=T> + Sum,
    F1: Fn(P) -> T,
    F2: Fn(P) -> T,
{
    Seq::gen_seq(&lb, &ub).map(|y| f1(x.clone()-y.clone())*f2(y)).sum()
}

pub trait NumRandVar<P, T>: RandVar<P, T>
where
    P: Seq + Zero + Add<P, Output=P> + Sub<P, Output=P>,
    T: Zero + One + Sum + Add<T, Output=T> + Sub<T, Output=T> + Mul<T, Output=T> + Clone,
{
    fn convert(&self, p: P) -> T;

    fn expected_value(&self) -> T {
        self.general_expected_value(|p| self.convert(p))
    }

    fn variance(&self) -> T {
        let ev = self.expected_value();
        let sq_ev = self.general_expected_value(|p| num::pow(self.convert(p),2));
        sq_ev - num::pow(ev, 2)
    }

    fn print_stats(&self)
    where
        T: Display,
    {
        println!("ev  = {}", self.expected_value());
        println!("var = {}", self.variance());
    }

    fn print_stats_convert<F,T2>(&self, f: F)
    where
        T: Display,
        F: Fn(T) -> T2,
        T2: Display,
    {
        println!("ev  = {} ~= {}", self.expected_value(), f(self.expected_value()));
        println!("var = {} ~= {}", self.variance(), f(self.variance()));
    }

    fn add_const(&self, p: P) -> Self
    where
        Self: Sized,
    {
        // .unwrap() is fine here, because if self is a valid RV, then this also will be.
        RandVar::build(
            SeqIter { items: self.valid_p().map(|x| x + p.clone()).collect() },
            |x| self.pdf(x-p.clone())
        ).unwrap()
    }

    fn convex_bounds(&self) -> (P, P) {
        if P::always_convex() {
            (self.lower_bound(), self.upper_bound())
        } else {
            P::convex_bounds(self.valid_p()).unwrap()
        }
    }

    fn add_rv(&self, other: &impl NumRandVar<P,T>) -> Self
    where
        Self: Sized
    {
        // care must be taken here that the bounds are convex for both RVs
        let (self_clb, self_cub) = self.convex_bounds();
        let (other_clb, other_cub) = other.convex_bounds();
        let new_lb = self_clb.clone() + other_clb.clone();
        let new_ub = self_cub.clone() + other_cub.clone();
        // this is only valid if '+' preserves convexity
        // which I think it should for sane '+' implementations
        let seq_iter = P::gen_seq(&new_lb, &new_ub);
        let min_lb;
        let max_ub;
        if P::always_convex() {
            min_lb = cmp::min(self_clb, other_clb);
            max_ub = cmp::max(self_cub, other_cub);
        } else {
            let set = BTreeSet::from([self_clb, other_clb, self_cub, other_cub]);
            let (lb, ub) = P::convex_bounds(SeqIter { items: set }).unwrap();
            min_lb = lb;
            max_ub = ub;
        }
        // .unwrap() is fine here, because if self and other are valid RVs, then this also will be.
        RandVar::build(
            seq_iter,
            |x| convolution(
                min_lb.clone(),
                max_ub.clone(),
                |p1| self.pdf(p1),
                |p2| other.pdf(p2),
                x)).unwrap()
    }

    fn multiple(&self, num_times: i32) -> Self
    where
        Self: Sized + Clone
    {
        if num_times == 0 {
            let seq_iter = SeqIter { items: BTreeSet::from([P::zero()]) };
            return RandVar::build(seq_iter, |_| T::one()).unwrap();
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
        let seq_iter = SeqIter { items: self.valid_p().map(|p| P::zero() - p).collect() };
        // .unwrap() is fine here, because if self is a valid RV, then this also will be.
        RandVar::build(
            seq_iter,
            |p| self.pdf(P::zero()-p)).unwrap()
    }

    fn half(&self) -> Result<Self, RVError>
    where
        P: One + Mul<P, Output=P> + Div<P, Output=P>,
        Self: Sized
    {
        let two = P::one() + P::one();
        let seq_iter = SeqIter { items: self.valid_p().map(|p| p/two.clone()).collect() };
        // check the way rounding works.
        if P::one()/two.clone() == P::zero() {
            // division truncates, so handle all collisions (i.e.: 2/2 == 3/2 == 1)
            RandVar::build(seq_iter, |p| {
                if p > P::zero() {
                    self.pdf(two.clone()*p.clone()) + self.pdf(two.clone()*p + P::one())
                } else if p == P::zero() {
                    self.pdf(P::zero()-P::one()) + self.pdf(P::zero()) + self.pdf(P::one())
                } else {
                    self.pdf(two.clone()*p.clone()) + self.pdf(two.clone()*p - P::one())
                }
            })
        } else if sanity_check::<P>() {
            // division is "rational-like", so there are no collisions.
            RandVar::build(seq_iter, |p| self.pdf(two.clone()*p))
        } else {
            // Your numbers are garbage. It's your own fault for using a type like that.
            return Err(RVError::NoRound);
        }
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
