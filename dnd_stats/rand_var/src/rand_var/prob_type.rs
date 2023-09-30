use std::fmt::Debug;
use std::iter::Sum;
use std::ops::{Add, Mul, Sub};

use num::{FromPrimitive, Integer, One, Zero};
use num::rational::Ratio;

// The minimum collection of traits required of the probability-like type for the RandVar and NumRandVar traits
pub trait Prob: Zero + One + Sum + Add<Self, Output=Self> + Sub<Self, Output=Self> + Mul<Self, Output=Self> + Clone {}
impl<I: Integer + Clone> Prob for Ratio<I> {}

pub trait Reciprocal {
    fn reciprocal(&self) -> Option<Self> where Self: Sized;
}

impl<I: Integer + Clone> Reciprocal for Ratio<I> {
    fn reciprocal(&self) -> Option<Self> {
        if self.is_zero() {
            None
        } else {
            Some(Ratio::new(self.denom().clone(), self.numer().clone()))
        }
    }
}

// The collection of traits needed for all methods in the RandomVariable and MapRandVar structs
pub trait RVProb: Prob + PartialOrd<Self> + Ord + PartialEq<Self> + FromPrimitive + Reciprocal + for<'a> Sum<&'a Self> + Debug {}
impl<I: Integer + Clone + Debug> RVProb for Ratio<I> where Ratio<I>: FromPrimitive {}
