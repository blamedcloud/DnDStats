use std::iter::Sum;
use std::ops::{Add, Mul, Sub};
use num::{Integer, One, Zero};
use num::rational::Ratio;

pub trait ProbType: Zero + One + Sum + Add<Self, Output=Self> + Sub<Self, Output=Self> + Mul<Self, Output=Self> + Clone {}

impl<T> ProbType for Ratio<T>
where
    T: Integer + Clone
{}
