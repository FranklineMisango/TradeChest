use std::ops::{Add, Sub, Mul, Div};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FixedPoint(i64);

const SCALE: i64 = 1_000_000; // 6 decimal places

impl FixedPoint {
    pub fn from_f64(value: f64) -> Self {
        Self((value * SCALE as f64) as i64)
    }
    
    pub fn to_f64(self) -> f64 {
        self.0 as f64 / SCALE as f64
    }
    
    pub fn zero() -> Self {
        Self(0)
    }
    
    pub fn is_positive(self) -> bool {
        self.0 > 0
    }
}

impl Add for FixedPoint {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub for FixedPoint {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Mul for FixedPoint {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self((self.0 * rhs.0) / SCALE)
    }
}

impl Div for FixedPoint {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        Self((self.0 * SCALE) / rhs.0)
    }
}

impl From<f64> for FixedPoint {
    fn from(value: f64) -> Self {
        Self::from_f64(value)
    }
}

impl From<FixedPoint> for f64 {
    fn from(value: FixedPoint) -> Self {
        value.to_f64()
    }
}