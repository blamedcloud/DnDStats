use std::ops::Add;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub struct Feet(pub i32);
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
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

impl From<Square> for Feet {
    fn from(value: Square) -> Self {
        Feet(value.0 * 5)
    }
}
