use std::cmp::Ord;
use std::cmp::Ordering;
use std::cmp::PartialOrd;
use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Sub;
use std::ops::SubAssign;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CascadeOrd {
    pub importance: Importance,
    pub specificity: Specificity,
}

impl CascadeOrd {
    pub const ID: Specificity = Specificity::ID;
    pub const CLASS: Specificity = Specificity::CLASS;
    pub const TYPE: Specificity = Specificity::TYPE;
    pub const NONE: Specificity = Specificity::NONE;

    pub fn new(origin: Origin, important: bool) -> Self {
        Self {
            importance: Importance { origin, important },
            specificity: Specificity::NONE,
        }
    }
}

impl AddAssign<Specificity> for CascadeOrd {
    fn add_assign(&mut self, lhs: Specificity) {
        self.specificity += lhs;
    }
}

impl Add<Specificity> for CascadeOrd {
    type Output = Self;

    fn add(mut self, lhs: Specificity) -> Self {
        self += lhs;
        self
    }
}

impl SubAssign<Specificity> for CascadeOrd {
    fn sub_assign(&mut self, lhs: Specificity) {
        self.specificity -= lhs;
    }
}

impl Sub<Specificity> for CascadeOrd {
    type Output = Self;

    fn sub(mut self, lhs: Specificity) -> Self {
        self -= lhs;
        self
    }
}

impl PartialOrd for CascadeOrd {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(
            self.importance
                .cmp(&other.importance)
                .then(self.specificity.cmp(&other.specificity)),
        )
    }
}

impl Ord for CascadeOrd {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Importance {
    pub origin: Origin,
    pub important: bool,
}

impl Importance {
    fn as_ord(&self) -> u32 {
        match self.origin {
            Origin::UserAgent => {
                if !self.important {
                    1
                } else {
                    7
                }
            }
            Origin::User => {
                if !self.important {
                    2
                } else {
                    6
                }
            }
            Origin::Developer => {
                if !self.important {
                    3
                } else {
                    5
                }
            }
            Origin::KeyframesAnimation => 4,
            Origin::Transition => 8,
        }
    }
}

impl PartialOrd for Importance {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.as_ord().partial_cmp(&other.as_ord())
    }
}

impl Ord for Importance {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Origin {
    UserAgent,
    User,
    Developer,
    KeyframesAnimation,
    Transition,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Specificity(u32, u32, u32);

impl Specificity {
    pub const ID: Self = Self(1, 0, 0);
    pub const CLASS: Self = Self(0, 1, 0);
    pub const TYPE: Self = Self(0, 0, 1);
    pub const NONE: Self = Self(0, 0, 0);
}

impl AddAssign<Self> for Specificity {
    fn add_assign(&mut self, rhs: Self) {
        self.0 = self.0.saturating_add(rhs.0);
        self.1 = self.1.saturating_add(rhs.1);
        self.2 = self.2.saturating_add(rhs.2);
    }
}

impl Add<Self> for Specificity {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self {
        self += rhs;
        self
    }
}

impl SubAssign<Self> for Specificity {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 = self.0.saturating_sub(rhs.0);
        self.1 = self.1.saturating_sub(rhs.1);
        self.2 = self.2.saturating_sub(rhs.2);
    }
}

impl Sub<Self> for Specificity {
    type Output = Self;

    fn sub(mut self, rhs: Self) -> Self {
        self -= rhs;
        self
    }
}

impl PartialOrd for Specificity {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(
            self.0
                .cmp(&other.0)
                .then(self.1.cmp(&other.1))
                .then(self.2.cmp(&other.2)),
        )
    }
}

impl Ord for Specificity {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}
