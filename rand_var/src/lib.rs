use std::fmt::Debug;
use std::iter::Sum;
use num::{FromPrimitive, Integer, Num, One, PrimInt};
use num::rational::Ratio;

pub struct RandomVariable<T: Num> {
    lower_bound: isize,
    upper_bound: isize,
    pdf_vec: Vec<T>,
}

impl<T> RandomVariable<Ratio<T>>
where
    T: Integer + Debug + Clone + FromPrimitive
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
    T: Num + Sum + Debug + Clone
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

    unsafe fn raw_pdf(&self, x: isize) -> T {
        assert!(self.lower_bound <= x);
        assert!(x <= self.upper_bound);
        let index: usize = (x - self.lower_bound) as usize;
        self.pdf_vec.get(index).unwrap().clone()
    }

    fn valid_p(&self) -> Box<dyn Iterator<Item=isize>> {
        Box::new(self.lower_bound..=self.upper_bound)
    }
}

impl<T> NumRandVar<isize, T> for RandomVariable<T>
where
    T: Num + Sum + Debug + Clone + FromPrimitive
{
    fn convert(&self, p: isize) -> T {
        T::from_isize(p).unwrap()
    }
}

pub trait RandVar<P, T>
where
    P: Ord,
    T: Num + Sum,
{
    fn build<F: Fn(P) -> T>(lb: P, ub: P, f: F) -> Self;
    fn lower_bound(&self) -> P;
    fn upper_bound(&self) -> P;
    unsafe fn raw_pdf(&self, p: P) -> T;
    fn valid_p(&self) -> Box<dyn Iterator<Item=P>>;

    fn pdf(&self, p: P) -> T {
        if (self.lower_bound() <= p) && (p <= self.upper_bound()) {
            unsafe {
                self.raw_pdf(p)
            }
        } else {
            T::zero()
        }
    }

    fn cdf(&self, p: P) -> T {
        if self.upper_bound() <= p {
            T::one()
        } else if self.lower_bound() <= p {
            self.valid_p().take_while(|x| x <= &p).map(|x| self.pdf(x)).sum()
        } else {
            T::zero()
        }
    }
}

fn convolution<P, T, F>(lb: P, ub: P, f1: F, f2: F, x: P) -> T
where
    P: PrimInt,
    T: Num + Sum,
    F: Fn(P) -> T,
{
    num::range_inclusive(lb, ub).map(|y| f1(x-y)*f2(y)).sum()
}

pub trait NumRandVar<P, T>: RandVar<P, T>
where
    P: PrimInt,
    T: Num + Sum + Clone,
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
        for p in num::range_inclusive(self.lower_bound(), self.upper_bound()) {
            result = result + f(p) * self.pdf(p);
        }
        result
    }

    fn variance(&self) -> T {
        let ev = self.expected_value();
        let sq_ev = self.general_expected_value(|p| num::pow(self.convert(p),2));
        sq_ev - num::pow(ev, 2)
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
