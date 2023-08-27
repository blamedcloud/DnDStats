use std::ops::Add;

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Clone, Copy)]
pub struct Feet(pub i32);
#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Clone, Copy)]
pub struct Square(pub i32);

impl Add<Square> for Square {
    type Output = Square;

    fn add(self, other: Square) -> Square {
        Square(self.0 + other.0)
    }
}

impl Add<Square> for Feet {
    type Output = Feet;

    fn add(self, other: Square) -> Feet {
        Feet(self.0 + (other.0 * 5))
    }
}

impl Add<Feet> for Feet {
    type Output = Feet;

    fn add(self, other: Feet) -> Feet {
        Feet(self.0 + other.0)
    }
}