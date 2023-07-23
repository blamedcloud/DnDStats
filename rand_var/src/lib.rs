use std::cmp;
use std::fmt::{Debug, Display};
use std::iter::Sum;
use num::{FromPrimitive, Integer, Num, One, PrimInt};
use num::rational::Ratio;

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
        for i in lb..=ub {
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

    fn valid_p(&self) -> Box<dyn Iterator<Item=isize>> {
        Box::new(self.lower_bound..=self.upper_bound)
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

pub trait RandVar<P, T>
where
    P: Ord + Display,
    T: Num + Sum + Display,
{
    fn build<F: Fn(P) -> T>(lb: P, ub: P, f: F) -> Self;
    fn lower_bound(&self) -> P;
    fn upper_bound(&self) -> P;
    unsafe fn raw_pdf(&self, p: &P) -> T;
    fn valid_p(&self) -> Box<dyn Iterator<Item=P>>;

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

    fn cap_lb(&self, lb: P) -> Self
    where
        P: Copy,
        Self: Sized
    {
        // will panic if lb > ub
        RandVar::build(lb, self.upper_bound(), |p| {
            if p == lb {
                self.cdf(lb)
            } else {
                self.pdf(p)
            }
        })
    }

    fn cap_ub(&self, ub: P) -> Self
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

    fn max_two_trials(&self) -> Self
    where
        T: Clone,
        Self: Sized
    {
        let max_pdf = |p| {
            (T::one()+T::one()) * self.pdf_ref(&p) * self.cdf_exclusive_ref(&p) + num::pow(self.pdf_ref(&p), 2)
        };
        RandVar::build(self.lower_bound(), self.upper_bound(), max_pdf)
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
        RandVar::build(self.lower_bound(), self.upper_bound(), min_pdf)
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
        RandVar::build(self.lower_bound(), self.upper_bound(), max_pdf)
    }

    fn print_distributions(&self) {
        println!("P:\tpdf\t(cdf)");
        for p in self.valid_p() {
            println!("{}:\t{}\t({})", &p, self.pdf_ref(&p), self.cdf_ref(&p));
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
    P: PrimInt + Display,
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

    fn add_rv(&self, other: &Self) -> Self
    where
        Self: Sized
    {
        let new_lb = self.lower_bound() + other.lower_bound();
        let new_ub = self.upper_bound() + other.upper_bound();
        let min_lb = cmp::min(self.lower_bound(), other.lower_bound());
        let max_ub = cmp::max(self.upper_bound(), other.upper_bound());
        RandVar::build(
            new_lb,
            new_ub,
            |x| convolution(
                min_lb,
                max_ub,
                |p1| self.pdf(p1),
                |p2| other.pdf(p2),
                x))
    }

    fn multiple(&self, num_times: i32) -> Self
    where
        Self: Sized + Clone
    {
        if num_times == 0 {
            return RandVar::build(P::zero(), P::zero(), |_| T::one());
        } else if num_times == 1 {
            return self.clone();
        } else if num_times == -1 {
            return self.opposite_rv();
        }
        let is_neg = num_times < 0;
        let pos_num = num::abs(num_times);
        let mut rv = self.add_rv(&self);
        for _ in 2..pos_num {
            rv = rv.add_rv(&self);
        }
        if is_neg {
            rv.opposite_rv()
        } else {
            rv
        }
    }

    fn minus_rv(&self, other: &Self) -> Self
    where
        Self: Sized
    {
        self.add_rv(&other.opposite_rv())
    }

    fn opposite_rv(&self) -> Self
    where
        Self: Sized
    {
        RandVar::build(
            P::zero()-self.upper_bound(),
            P::zero()-self.lower_bound(),
            |p| self.pdf(P::zero()-p))
    }

    fn half(&self) -> Self
    where
        Self: Sized
    {
        let two = P::one() + P::one();
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

    fn prob_lt(&self, other: &Self) -> T
    where
        Self: Sized
    {
        self.minus_rv(other).cdf_exclusive(P::zero())
    }

    fn prob_le(&self, other: &Self) -> T
    where
        Self: Sized
    {
        self.minus_rv(other).cdf(P::zero())
    }

    fn prob_eq(&self, other: &Self) -> T
    where
        Self: Sized
    {
        self.minus_rv(other).pdf(P::zero())
    }

    fn prob_gt(&self, other: &Self) -> T
    where
        Self: Sized
    {
        let diff_rv = self.minus_rv(other);
        T::one() - diff_rv.cdf(P::zero())
    }

    fn prob_ge(&self, other: &Self) -> T
    where
        Self: Sized
    {
        let diff_rv = self.minus_rv(other);
        T::one() - diff_rv.cdf_exclusive(P::zero())
    }
}


#[cfg(test)]
mod tests {
    use num::{BigInt, BigRational, Rational64, Zero};
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
    fn test_minus_d4() {
        let rv: RandomVariable<Rational64> = RandomVariable::new_dice(4).opposite_rv();
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
        let rv1: RandomVariable<Rational64> = RandomVariable::new_dice(20);
        let rv2: RandomVariable<Rational64> = RandomVariable::new_constant(5);
        let rv = rv1.minus_rv(&rv2).cap_lb(0);
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
        let rv1: RandomVariable<Rational64> = RandomVariable::new_dice(11);
        let rv2: RandomVariable<Rational64> = RandomVariable::new_constant(3);
        let rv = rv1.add_rv(&rv2).cap_ub(10);
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
    }

    #[test]
    fn test_d12_minus_d8() {
        let rv1: RandomVariable<Rational64> = RandomVariable::new_dice(12);
        let rv2: RandomVariable<Rational64> = RandomVariable::new_dice(8);
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
        let rv1: RandomVariable<BigRational> = RandomVariable::new_dice(6);
        let rv2: RandomVariable<BigRational> = RandomVariable::new_dice(6);
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
        let rv1: RandomVariable<BigRational> = RandomVariable::new_dice(6);
        let rv = rv1.add_rv(&rv1);
        let other_rv = rv1.multiple(2);
        assert_eq!(rv, other_rv);

        let rv = rv1.add_rv(&rv1).add_rv(&rv1);
        let other_rv = rv1.multiple(3);
        assert_eq!(rv, other_rv);
    }

    #[test]
    fn test_fireball() {
        let d6: RandomVariable<BigRational> = RandomVariable::new_dice(6);
        let fireball = d6.multiple(8);
        assert_eq!(8, fireball.lower_bound());
        assert_eq!(48, fireball.upper_bound());
        assert_eq!(BigRational::from_i32(28).unwrap(), fireball.expected_value());

        let fireball_resist = fireball.half();
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
        let rv: RandomVariable<Rational64> = RandomVariable::new_uniform(-3,7).half();
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
        let rv: RandomVariable<Rational64> = RandomVariable::new_dice(20).max_two_trials();
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
        let rv: RandomVariable<Rational64> = RandomVariable::new_dice(20).min_two_trials();
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
        let rv: RandomVariable<Rational64> = RandomVariable::new_dice(20).max_three_trials();
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
        let d6: RandomVariable<Rational64> = RandomVariable::new_dice(6);
        assert_eq!(Rational64::new(5,12), d6.prob_gt(&d6));
        assert_eq!(Rational64::new(7,12), d6.prob_ge(&d6));
        assert_eq!(Rational64::new(1,6), d6.prob_eq(&d6));
        assert_eq!(Rational64::new(7,12), d6.prob_le(&d6));
        assert_eq!(Rational64::new(5,12), d6.prob_gt(&d6));

        let d20: RandomVariable<Rational64> = RandomVariable::new_dice(20);
        let d20_adv = d20.max_two_trials();
        assert_eq!(Rational64::one(), d20_adv.prob_ge(&d20) + d20_adv.prob_lt(&d20));
        assert_eq!(Rational64::one(), d20_adv.prob_gt(&d20) + d20_adv.prob_lt(&d20) + d20_adv.prob_eq(&d20));
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
