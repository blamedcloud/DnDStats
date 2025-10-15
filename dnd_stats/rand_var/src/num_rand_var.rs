use std::cmp;
use std::collections::BTreeSet;
use std::fmt::Display;
use std::iter::Sum;
use std::ops::{Add, Div, Mul, Sub};

use num::{One, ToPrimitive, Zero};

use crate::rand_var::prob_type::{Prob, two, three};
use crate::rand_var::RandVar;
use crate::rand_var::sequential::{Seq, SeqIter};
use crate::RVError;

pub trait NumRandVar<K, P>: RandVar<K, P>
where
    K: Seq + Zero + Add<K, Output=K> + Sub<K, Output=K>,
    P: Prob,
{
    fn convert(&self, k: K) -> P;

    fn expected_value(&self) -> P {
        self.general_expected_value(|k| self.convert(k))
    }

    fn variance(&self) -> P {
        let ev = self.expected_value();
        let sq_ev = self.general_expected_value(|k| num::pow(self.convert(k), 2));
        sq_ev - num::pow(ev, 2)
    }

    fn print_stats(&self)
    where
        P: Display,
    {
        println!("ev  = {}", self.expected_value());
        println!("var = {}", self.variance());
    }

    fn print_stats_f64(&self)
    where
        K: Display,
        P: Display + ToPrimitive,
    {
        let to_float = |p: P| p.to_f64().unwrap();
        self.print_stats_convert(&to_float);
        println!("std.dev ~= {}", to_float(self.variance()).sqrt());
    }

    fn print_stats_convert<F, P2>(&self, f: &F)
    where
        K: Display,
        P: Display,
        F: Fn(P) -> P2,
        P2: Display,
    {
        self.print_pdf(f);
        println!("Bounds: ({}, {})", self.lower_bound(), self.upper_bound());
        println!("ev  = {} ~= {}", self.expected_value(), f(self.expected_value()));
        println!("var = {} ~= {}", self.variance(), f(self.variance()));
    }

    fn add_const(&self, k: K) -> Self
    where
        Self: Sized,
    {
        // .unwrap() is fine here, because if self is a valid RV, then this also will be.
        RandVar::build(
            SeqIter { items: self.get_keys().map(|x| x + k.clone()).collect() },
            |x| self.pdf(x- k.clone())
        ).unwrap()
    }

    fn convex_bounds(&self) -> (K, K) {
        if K::always_convex() {
            (self.lower_bound(), self.upper_bound())
        } else {
            K::convex_bounds(self.get_keys()).unwrap()
        }
    }

    fn add_rv(&self, other: &impl NumRandVar<K, P>) -> Self
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
        let seq_iter = K::gen_seq(&new_lb, &new_ub);
        let min_lb;
        let max_ub;
        if K::always_convex() {
            min_lb = cmp::min(self_clb, other_clb);
            max_ub = cmp::max(self_cub, other_cub);
        } else {
            let set = BTreeSet::from([self_clb, other_clb, self_cub, other_cub]);
            let (lb, ub) = K::convex_bounds(SeqIter { items: set }).unwrap();
            min_lb = lb;
            max_ub = ub;
        }
        // .unwrap() is fine here, because if self and other are valid RVs, then this also will be.
        RandVar::build(
            seq_iter,
            |x| convolution(
                min_lb.clone(),
                max_ub.clone(),
                |k1| self.pdf(k1),
                |k2| other.pdf(k2),
                x)).unwrap()
    }

    fn multiple(&self, num_times: i32) -> Self
    where
        Self: Sized + Clone
    {
        if num_times == 0 {
            let seq_iter = SeqIter { items: BTreeSet::from([K::zero()]) };
            return RandVar::build(seq_iter, |_| P::one()).unwrap();
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

    fn minus_rv(&self, other: &impl NumRandVar<K, P>) -> Self
    where
        Self: Sized
    {
        self.add_rv(&other.opposite_rv())
    }

    fn opposite_rv(&self) -> Self
    where
        Self: Sized
    {
        let seq_iter = SeqIter { items: self.get_keys().map(|k| K::zero() - k).collect() };
        // .unwrap() is fine here, because if self is a valid RV, then this also will be.
        RandVar::build(
            seq_iter,
            |k| self.pdf(K::zero()- k)).unwrap()
    }

    fn half(&self) -> Result<Self, RVError>
    where
        K: One + Mul<K, Output=K> + Div<K, Output=K>,
        Self: Sized
    {
        let seq_iter = SeqIter { items: self.get_keys().map(|k| k / two::<K>()).collect() };
        // check the way rounding works.
        if K::one() / two::<K>() == K::zero() {
            // division truncates, so handle all collisions (i.e.: 2/2 == 3/2 == 1)
            RandVar::build(seq_iter, |k| {
                if k > K::zero() {
                    self.pdf(two::<K>() * k.clone()) + self.pdf(two::<K>() * k + K::one())
                } else if k == K::zero() {
                    self.pdf(K::zero() - K::one()) + self.pdf(K::zero()) + self.pdf(K::one())
                } else {
                    self.pdf(two::<K>() * k.clone()) + self.pdf(two::<K>() * k - K::one())
                }
            })
        } else if sanity_check::<K>() {
            // division is "rational-like", so there are no collisions.
            RandVar::build(seq_iter, |k| self.pdf(two::<K>() * k))
        } else {
            // Your numbers are garbage. It's your own fault for using a type like that.
            return Err(RVError::NoRound);
        }
    }

    fn prob_lt(&self, other: &impl NumRandVar<K, P>) -> P
    where
        Self: Sized
    {
        self.minus_rv(other).cdf_exclusive(K::zero())
    }

    fn prob_le(&self, other: &impl NumRandVar<K, P>) -> P
    where
        Self: Sized
    {
        self.minus_rv(other).cdf(K::zero())
    }

    fn prob_eq(&self, other: &impl NumRandVar<K, P>) -> P
    where
        Self: Sized
    {
        self.minus_rv(other).pdf(K::zero())
    }

    fn prob_gt(&self, other: &impl NumRandVar<K, P>) -> P
    where
        Self: Sized
    {
        let diff_rv = self.minus_rv(other);
        P::one() - diff_rv.cdf(K::zero())
    }

    fn prob_ge(&self, other: &impl NumRandVar<K, P>) -> P
    where
        Self: Sized
    {
        let diff_rv = self.minus_rv(other);
        P::one() - diff_rv.cdf_exclusive(K::zero())
    }
}

fn convolution<K, P, F1, F2>(lb: K, ub: K, f1: F1, f2: F2, x: K) -> P
    where
        K: Seq + Sub<K,Output=K>,
        P: Mul<P, Output=P> + Sum,
        F1: Fn(K) -> P,
        F2: Fn(K) -> P,
{
    Seq::gen_seq(&lb, &ub).map(|y| f1(x.clone()-y.clone())*f2(y)).sum()
}

// any sane numeric type ought to have 1/10 + 2/10 == 3/10
fn sanity_check<K>() -> bool
    where
        K: One + Add<K, Output=K> + Div<K, Output=K> + Clone + PartialEq
{
    let three = three::<K>();
    let ten = three.clone() + three.clone() + three.clone() + K::one();
    let left = (K::one()/ten.clone()) + (two::<K>()/ten.clone());
    let right = three/ten;
    left == right
}

#[cfg(test)]
mod tests {
    use num::{BigInt, BigRational, FromPrimitive, One, Rational64, Zero};
    use crate::map_rand_var::MRV64;
    use crate::num_rand_var::{sanity_check, NumRandVar};
    use crate::rand_var::RandVar;
    use crate::vec_rand_var::{VecRandVar, VRV64, VRVBig};

    #[test]
    fn test_minus_d4() {
        let rv: VRV64 = VecRandVar::new_dice(4).unwrap().opposite_rv();
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
        let rv1: VRV64 = VecRandVar::new_dice(20).unwrap();
        let rv2: VRV64 = VecRandVar::new_constant(5).unwrap();
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
        let rv1: VRV64 = VecRandVar::new_dice(11).unwrap();
        let rv2: VRV64 = VecRandVar::new_constant(3).unwrap();
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
        let rv1: VRV64 = VecRandVar::new_dice(12).unwrap();
        let rv2: VRV64 = VecRandVar::new_dice(8).unwrap();
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
    fn test_mixed_add() {
        let d8: VRV64 = VecRandVar::new_dice(8).unwrap();
        let const_5: MRV64 = VecRandVar::new_constant(5).unwrap().into();
        let d8_plus_5 = d8.add_rv(&const_5);
        assert_eq!(6, d8_plus_5.lower_bound());
        assert_eq!(13, d8_plus_5.upper_bound());
        assert_eq!(Rational64::new(19,2), d8_plus_5.expected_value());
        assert_eq!(d8_plus_5, d8.add_const(5));
    }

    #[test]
    fn test_2d6() {
        let rv1: VRVBig = VecRandVar::new_dice(6).unwrap();
        let rv2: VRVBig = VecRandVar::new_dice(6).unwrap();
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
        let rv1: VRVBig = VecRandVar::new_dice(6).unwrap();
        let rv = rv1.add_rv(&rv1);
        let other_rv = rv1.multiple(2);
        assert_eq!(rv, other_rv);

        let rv = rv1.add_rv(&rv1).add_rv(&rv1);
        let other_rv = rv1.multiple(3);
        assert_eq!(rv, other_rv);
    }

    #[test]
    fn test_fireball() {
        let d6: VRVBig = VecRandVar::new_dice(6).unwrap();
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
        let rv: VRV64 = VecRandVar::new_uniform(-3, 7).unwrap().half().unwrap();
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
    fn test_sanity_check() {
        assert_eq!(true, sanity_check::<Rational64>());
        assert_eq!(true, sanity_check::<BigRational>());
        //assert_eq!(false, sanity_check::<f32>()); // TODO: fix this
        assert_eq!(false, sanity_check::<f64>());

    }
}
