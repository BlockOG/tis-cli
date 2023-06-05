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

impl From<i8> for Number {
    fn from(number: i8) -> Self {
        Self(number as i16)
    }
}

impl From<u8> for Number {
    fn from(number: u8) -> Self {
        Self(number as i16)
    }
}

macro_rules! impl_from_signed {
    ($($type:ty),*) => {
        $(
            impl From<$type> for Number {
                fn from(number: $type) -> Self {
                    Self(number.clamp(-999, 999) as i16)
                }
            }
        )*
    };
}

macro_rules! impl_from_unsigned {
    ($($type:ty),*) => {
        $(
            impl From<$type> for Number {
                fn from(number: $type) -> Self {
                    Self(number.min(999) as i16)
                }
            }
        )*
    };
}

impl_from_signed!(i16, i32, i64, i128, isize);
impl_from_unsigned!(u16, u32, u64, u128, usize);

impl AddAssign for Number {
    fn add_assign(&mut self, rhs: Self) {
        self.set_value(self.value() + rhs.value());
    }
}

impl SubAssign for Number {
    fn sub_assign(&mut self, rhs: Self) {
        self.set_value(self.value() - rhs.value());
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
        self.value() == 0
    }
}

impl FromStr for Number {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut s = s.chars().peekable();
        let mut value = 0;

        let is_negative = s.peek().ok_or("Empty string".to_owned())? == &'-';
        if is_negative {
            s.next();
        }

        while let Some(c) = s.next() {
            match c {
                '0'..='9' => {
                    value *= 10;
                    value += c.to_digit(10).unwrap() as i16;
                    value = value.min(999);
                }

                _ => return Err(format!("Invalid digit: '{}'", c)),
            }
        }

        Ok(if is_negative {
            Self::from(-value)
        } else {
            Self::from(value)
        })
    }
}

impl ToString for Number {
    fn to_string(&self) -> String {
        self.value().to_string()
    }
}
