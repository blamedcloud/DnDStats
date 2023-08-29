use std::cmp::Ordering;
use std::ops::{Add, AddAssign, Div, Sub, SubAssign};

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum RefreshBy {
    Const(usize),
    ToFull,
    ToEmpty,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum ResourceCap {
    Soft(usize),
    Hard(usize),
    UnCapped,
}

impl ResourceCap {
    pub fn cap(&self) -> Option<usize> {
        match self {
            ResourceCap::Soft(c) => Some(*c),
            ResourceCap::Hard(c) => Some(*c),
            ResourceCap::UnCapped => None,
        }
    }

    pub fn is_uncapped(&self) -> bool {
        match self {
            ResourceCap::UnCapped => true,
            _ => false,
        }
    }
}

impl Add<ResourceCap> for ResourceCap {
    type Output = ResourceCap;

    fn add(self, rhs: ResourceCap) -> Self::Output {
        match self {
            ResourceCap::Soft(c) => {
                match rhs {
                    ResourceCap::Soft(r) => ResourceCap::Soft(c+r),
                    ResourceCap::Hard(r) => ResourceCap::Soft(c+r),
                    ResourceCap::UnCapped => ResourceCap::UnCapped
                }
            },
            ResourceCap::Hard(c) => {
                match rhs {
                    ResourceCap::Soft(r) => ResourceCap::Soft(c+r),
                    ResourceCap::Hard(r) => ResourceCap::Hard(c+r),
                    ResourceCap::UnCapped => ResourceCap::UnCapped
                }
            },
            ResourceCap::UnCapped => ResourceCap::UnCapped
        }
    }
}
impl Div<usize> for ResourceCap {
    type Output = Option<ResourceCap>;

    fn div(self, rhs: usize) -> Self::Output {
        if rhs == 0 {
            None
        } else {
            match self {
                ResourceCap::Soft(c) => Some(ResourceCap::Soft(c/2)),
                ResourceCap::Hard(c) => Some(ResourceCap::Hard(c/2)),
                ResourceCap::UnCapped => Some(ResourceCap::UnCapped)
            }
        }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum ResourceCount {
    Count(usize),
    UnCapped,
}

impl ResourceCount {
    pub fn count(&self) -> Option<usize> {
        match self {
            ResourceCount::Count(c) => Some(*c),
            ResourceCount::UnCapped => None
        }
    }

    pub fn set_count(&mut self, count: usize) {
        if let ResourceCount::Count(c) = self {
            *c = count;
        }
    }

    pub fn is_uncapped(&self) -> bool {
        match self {
            ResourceCount::UnCapped => true,
            _ => false,
        }
    }
}

impl Add<ResourceCount> for ResourceCount {
    type Output = Self;

    fn add(self, rhs: ResourceCount) -> Self::Output {
        match self {
            ResourceCount::Count(l) => {
                match rhs {
                    ResourceCount::Count(r) => ResourceCount::Count(l+r),
                    ResourceCount::UnCapped => ResourceCount::UnCapped
                }
            }
            ResourceCount::UnCapped => ResourceCount::UnCapped
        }
    }
}
impl Add<usize> for ResourceCount {
    type Output = ResourceCount;

    fn add(self, rhs: usize) -> Self::Output {
        match self {
            ResourceCount::Count(c) => ResourceCount::Count(c + rhs),
            ResourceCount::UnCapped => ResourceCount::UnCapped,
        }
    }
}
impl AddAssign<usize> for ResourceCount {
    fn add_assign(&mut self, rhs: usize) {
        match self {
            ResourceCount::Count(c) => *c += rhs,
            ResourceCount::UnCapped => {}
        }
    }
}

impl Sub<usize> for ResourceCount {
    type Output = ResourceCount;

    fn sub(self, rhs: usize) -> Self::Output {
        match self {
            ResourceCount::Count(c) => {
                if rhs >= c {
                    ResourceCount::Count(0)
                } else {
                    ResourceCount::Count(c-rhs)
                }
            }
            ResourceCount::UnCapped => ResourceCount::UnCapped,
        }
    }
}
impl SubAssign<usize> for ResourceCount {
    fn sub_assign(&mut self, rhs: usize) {
        match self {
            ResourceCount::Count(c) => {
                if &rhs >= c {
                    *c = 0;
                } else {
                    *c -= rhs
                }
            },
            ResourceCount::UnCapped => {}
        }
    }
}

impl PartialEq<ResourceCap> for ResourceCount {
    fn eq(&self, other: &ResourceCap) -> bool {
        match self {
            ResourceCount::Count(cap) => {
                !other.is_uncapped() && cap == &other.cap().unwrap()
            },
            ResourceCount::UnCapped => other.is_uncapped()
        }
    }
}
impl PartialEq<ResourceCount> for ResourceCap {
    fn eq(&self, other: &ResourceCount) -> bool {
        other.eq(self)
    }
}
impl PartialEq<usize> for ResourceCount {
    fn eq(&self, other: &usize) -> bool {
        match self {
            ResourceCount::Count(c) => c == other,
            ResourceCount::UnCapped => false
        }
    }
}
impl PartialOrd<usize> for ResourceCount {
    fn partial_cmp(&self, other: &usize) -> Option<Ordering> {
        match self {
            ResourceCount::Count(c) => c.partial_cmp(other),
            ResourceCount::UnCapped => Some(Ordering::Greater)
        }
    }
}

impl From<ResourceCap> for ResourceCount {
    fn from(value: ResourceCap) -> Self {
        match value {
            ResourceCap::Soft(c) => ResourceCount::Count(c),
            ResourceCap::Hard(c) => ResourceCount::Count(c),
            ResourceCap::UnCapped => ResourceCount::UnCapped
        }
    }
}