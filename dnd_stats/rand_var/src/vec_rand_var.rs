use std::iter::Sum;

use num::{BigRational, FromPrimitive, Rational64};

use crate::map_rand_var::MapRandVar;
use crate::num_rand_var::NumRandVar;
use crate::rand_var::prob_type::{Prob, Reciprocal};
use crate::rand_var::RandVar;
use crate::rand_var::sequential::{Seq, SeqIter};
use crate::RVError;

#[derive(PartialEq, Clone, Debug)]
pub struct VecRandVar<P: Prob> {
    lower_bound: isize,
    upper_bound: isize,
    pdf_vec: Vec<P>,
}

pub type VRV64 = VecRandVar<Rational64>;
pub type VRVBig = VecRandVar<BigRational>;

impl<P> VecRandVar<P>
where
    P: Prob + PartialEq<P> + for<'a> Sum<&'a P> + Ord
{
    pub fn new(lb: isize, ub: isize, v: Vec<P>) -> Result<Self, RVError> {
        if lb > ub {
            return Err(RVError::InvalidBounds);
        }
        if (ub-lb+1) != (v.len() as isize) {
            return Err(RVError::Other(String::from("vector must be of correct length")));
        }
        if P::one() != v.iter().sum() {
            return Err(RVError::CDFNotOne);
        }
        if v.iter().min().unwrap() < &P::zero() {
            return Err(RVError::NegProb);
        }
        Ok(Self {
            lower_bound: lb,
            upper_bound: ub,
            pdf_vec: v
        })
    }

    pub fn backing_vec(&self) -> &Vec<P> {
        &self.pdf_vec
    }
}

impl<P> VecRandVar<P>
where
    P: Prob + Reciprocal + FromPrimitive
{
    pub fn new_dice(sides: isize) -> Result<Self, RVError> {
        if sides < 1 {
            return Err(RVError::InvalidBounds);
        }
        let mut v = Vec::with_capacity(sides as usize);
        let p = P::from_isize(sides).unwrap().reciprocal().unwrap();
        for _ in 0..sides {
            v.push(p.clone());
        }
        Ok(Self {
            lower_bound: 1,
            upper_bound: sides,
            pdf_vec: v
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
        let one_over_sides = P::from_isize(sides).unwrap().reciprocal().unwrap();
        let reroll_over_sides = P::from_isize(reroll_max).unwrap() * one_over_sides.clone();
        let reroll_only = reroll_over_sides * one_over_sides.clone();
        let possible_reroll = one_over_sides + reroll_only.clone();
        let mut v = Vec::with_capacity(sides as usize);
        for i in 1..=sides {
            if i > reroll_max {
                v.push(possible_reroll.clone());
            } else {
                v.push(reroll_only.clone());
            }
        }
        Ok(Self {
            lower_bound: 1,
            upper_bound: sides,
            pdf_vec: v
        })
    }

    pub fn new_constant(value: isize) -> Result<Self, RVError> {
        Ok(Self {
            lower_bound: value,
            upper_bound: value,
            pdf_vec: vec!(P::one())
        })
    }

    pub fn new_uniform(lb: isize, ub: isize) -> Result<Self, RVError> {
        if ub < lb {
            return Err(RVError::InvalidBounds);
        }
        let size = (ub - lb + 1) as usize;
        let mut v = Vec::with_capacity(size);
        let p = P::from_usize(size).unwrap().reciprocal().unwrap();
        for _ in 0..size {
            v.push(p.clone());
        }
        Ok(Self {
            lower_bound: lb,
            upper_bound: ub,
            pdf_vec: v
        })
    }
}

impl<P> RandVar<isize, P> for VecRandVar<P>
where
    P: Prob + PartialOrd<P>
{
    fn build<F>(seq_iter: SeqIter<isize>, f: F) -> Result<Self, RVError>
    where
        F: Fn(isize) -> P
    {
        if seq_iter.items.len() == 0 {
            return Err(RVError::InvalidBounds);
        }
        let lb = seq_iter.items.first().unwrap().clone();
        let ub = seq_iter.items.last().unwrap().clone();
        let mut pdf_vec = Vec::with_capacity((ub - lb + 1) as usize);
        let mut total = P::zero();
        for i in lb..=ub {
            if seq_iter.items.contains(&i) {
                let f_i = f(i);
                if f_i < P::zero() {
                    return Err(RVError::NegProb);
                }
                total = total + f_i.clone();
                pdf_vec.push(f_i);
            } else {
                pdf_vec.push(P::zero());
            }
        }
        if P::one() != total {
            return Err(RVError::CDFNotOne);
        }

        Ok(Self {
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

    unsafe fn raw_pdf(&self, k: &isize) -> P {
        assert!(self.lower_bound <= *k);
        assert!(*k <= self.upper_bound);
        let index: usize = (k - self.lower_bound) as usize;
        self.pdf_vec.get(index).unwrap().clone()
    }

    fn get_keys(&self) -> SeqIter<isize> {
        Seq::gen_seq(&self.lower_bound(), &self.upper_bound())
    }

    fn len(&self) -> usize {
        self.pdf_vec.len()
    }
}

impl<P> NumRandVar<isize, P> for VecRandVar<P>
where
    P: Prob + PartialOrd<P> + FromPrimitive
{
    fn convert(&self, k: isize) -> P {
        P::from_isize(k).unwrap()
    }
}

impl<P: Prob> From<VecRandVar<P>> for Vec<P> {
    fn from(value: VecRandVar<P>) -> Self {
        value.pdf_vec
    }
}

impl<P: Prob> From<MapRandVar<isize, P>> for VecRandVar<P> {
    fn from(value: MapRandVar<isize, P>) -> Self {
        let lb = *value.get_lb();
        let ub = *value.get_ub();
        let mut pdf_vec = Vec::with_capacity((ub-lb+1) as usize);
        for i in Seq::gen_seq(&lb, &ub) {
            if let Some(p) = value.backing_map().get(&i) {
                pdf_vec.push(p.clone());
            } else {
                pdf_vec.push(P::zero());
            }
        }
        // trust that value is a valid MapRandVar,
        // so this will be a valid RandomVariable.
        Self {
            lower_bound: lb,
            upper_bound: ub,
            pdf_vec
        }
    }
}

impl<P: Prob + PartialOrd<P>> VecRandVar<P> {
    pub fn into_mrv(self) -> MapRandVar<isize, P> {
        self.into()
    }
}

#[cfg(test)]
mod tests {
    use num::{One, Rational64, Zero};
    use crate::num_rand_var::NumRandVar;
    use crate::rand_var::RandVar;
    use crate::vec_rand_var::{VecRandVar, VRV64};

    #[test]
    fn test_const() {
        let rv: VRV64 = VecRandVar::new_constant(5).unwrap();
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
        let rv: VRV64 = VecRandVar::new_constant(-7).unwrap();
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
        let rv: VRV64 = VecRandVar::new_uniform(3, 14).unwrap();
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
        let rv: VRV64 = VecRandVar::new_dice(6).unwrap();
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
        let rv: VRV64 = VecRandVar::new_dice_reroll(10, 2).unwrap();
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
