use std::fmt::Debug;
use num::{FromPrimitive, Num, PrimInt};

pub struct RandomVariable<T: Num> {
    lower_bound: isize,
    upper_bound: isize,
    pdf_vec: Vec<T>,
}

impl<T: Num + Debug> RandomVariable<T> {

    pub fn build<F>(lb: isize, ub: isize, f: F) -> RandomVariable<T>
    where
        F: Fn(isize) -> T
    {
        assert!(lb <= ub);
        let length: usize = (ub - lb + 1) as usize;
        let mut pdf_vec = Vec::with_capacity(length);
        let mut total = T::zero();
        for i in lb..=ub {
            total = total + f(i);
            pdf_vec.push(f(i));
        }
        assert_eq!(T::one(), total);

        RandomVariable {
            lower_bound: lb,
            upper_bound: ub,
            pdf_vec
        }
    }
}

impl<T> RandVar<isize, T> for RandomVariable<T>
where
    T: Num + Debug + Clone
{
    fn lower_bound(&self) -> isize {
        self.lower_bound
    }

    fn upper_bound(&self) -> isize {
        self.upper_bound
    }

    fn raw_pdf(&self, x: isize) -> T {
        assert!(self.lower_bound <= x);
        assert!(x <= self.upper_bound);
        let index: usize = (x - self.lower_bound) as usize;
        self.pdf_vec.get(index).unwrap().clone()
    }
}

impl<T> NumRandVar<isize, T> for RandomVariable<T>
    where
        T: Num + Debug + Clone + FromPrimitive
{
    fn convert(&self, p: isize) -> T {
        T::from_isize(p).unwrap()
    }
}


pub trait RandVar<P, T>
where
    P: Ord,
    T: Num,
{
    fn lower_bound(&self) -> P;
    fn upper_bound(&self) -> P;

    fn raw_pdf(&self, p: P) -> T;
    fn pdf(&self, p: P) -> T {
        if (self.lower_bound() <= p) && (p <= self.upper_bound()) {
            self.raw_pdf(p)
        } else {
            T::zero()
        }
    }
}

fn summation<P, T, F>(lb: P, ub: P, f: F) -> T
    where
        P: PrimInt,
        T: Num,
        F: Fn(P) -> T,
{
    let mut result = T::zero();
    for p in num::range_inclusive(lb, ub) {
        result = result + f(p);
    }
    result
}

fn convolution<P, T, F>(lb: P, ub: P, f1: F, f2: F, x: P) -> T
    where
        P: PrimInt,
        T: Num,
        F: Fn(P) -> T,
{
    summation(lb, ub, |y| f1(x-y)*f2(y))
}

pub trait NumRandVar<P, T>: RandVar<P, T>
    where
        P: PrimInt,
        T: Num + Clone,
{
    fn cdf(&self, p: P) -> T {
        if self.upper_bound() <= p {
            T::one()
        } else if self.lower_bound() <= p {
            summation(self.lower_bound(), p, |x| self.pdf(x))
        } else {
            T::zero()
        }
    }

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