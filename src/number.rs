use std::{
    ops::{Add, AddAssign, Neg, Sub, SubAssign},
    str::FromStr,
};

use num_traits::Zero;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Number(i16);

impl Number {
    pub(crate) fn new() -> Self {
        Self(0)
    }

    pub(crate) fn value(&self) -> i16 {
        self.0
    }

    pub(crate) fn set_value(&mut self, value: i16) {
        self.0 = value.clamp(-999, 999);
    }
}

impl From<i16> for Number {
    fn from(number: i16) -> Self {
        Self(number.clamp(-999, 999))
    }
}

impl From<i32> for Number {
    fn from(number: i32) -> Self {
        Self(number.clamp(-999, 999) as i16)
    }
}

impl AddAssign for Number {
    fn add_assign(&mut self, rhs: Self) {
        self.set_value(self.0 + rhs.value());
    }
}

impl SubAssign for Number {
    fn sub_assign(&mut self, rhs: Self) {
        self.set_value(self.0 - rhs.value());
    }
}

impl Add for Number {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::from(self.value() + rhs.value())
    }
}

impl Sub for Number {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::from(self.value() - rhs.value())
    }
}

impl Neg for Number {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::from(-self.value())
    }
}

impl Zero for Number {
    fn zero() -> Self {
        Self::new()
    }

    fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

impl FromStr for Number {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut s = s.chars();
        let mut value = 0;

        if let Some(c) = s.next() {
            match c {
                '-' => value *= -1,
                '0'..='9' => {
                    value *= 10;
                    value += c.to_digit(10).unwrap() as i16;
                }

                _ => return Err(format!("Invalid digit: '{}'", c)),
            }
        }
        while let Some(c) = s.next() {
            match c {
                '0'..='9' if -999 < value && value < 999 => {
                    value *= 10;
                    value += c.to_digit(10).unwrap() as i16;
                    value = value.clamp(-999, 999);
                }
                '0'..='9' => {}

                _ => return Err(format!("Invalid digit: '{}'", c)),
            }
        }

        Ok(Self::from(value))
    }
}

impl ToString for Number {
    fn to_string(&self) -> String {
        self.value().to_string()
    }
}
