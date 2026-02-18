/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

#[cfg(test)]
#[path = "test/fct.rs"]
mod tests;

#[macro_export]
macro_rules! fct {
    ($($t:tt)*) => {
        $crate::fct::FCT(rust_decimal_macros::dec!($($t)*))
    };
}

use rust_decimal::prelude::FromPrimitive;
use rust_decimal::{Decimal, RoundingStrategy};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct FCT(pub Decimal);

impl FCT {
    pub fn new(val: Decimal) -> Self {
        Self(val)
    }

    pub fn to_str(&self, precision: usize) -> String {
        if precision == 0 {
            self.0.normalize().to_string()
        } else {
            let val = self.0.round_dp_with_strategy(precision as u32, RoundingStrategy::ToZero);
            format!("{:.*}", precision, val)
        }
    }
}

impl From<Decimal> for FCT {
    fn from(val: Decimal) -> Self {
        FCT(val)
    }
}

impl From<FCT> for Decimal {
    fn from(val: FCT) -> Decimal {
        val.0
    }
}

impl std::ops::Deref for FCT {
    type Target = Decimal;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for FCT {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::fmt::Display for FCT {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

macro_rules! impl_op {
    ($trait:ident, $method:ident) => {
        impl std::ops::$trait for FCT {
            type Output = Self;
            fn $method(self, rhs: Self) -> Self::Output {
                Self(self.0.$method(rhs.0))
            }
        }
    };
}

macro_rules! impl_op_assign {
    ($trait:ident, $method:ident) => {
        impl std::ops::$trait for FCT {
            fn $method(&mut self, rhs: Self) {
                self.0.$method(rhs.0)
            }
        }
    };
}

impl_op!(Add, add);
impl_op!(Sub, sub);
impl_op!(Mul, mul);
impl_op!(Div, div);
impl_op!(Rem, rem);

impl_op_assign!(AddAssign, add_assign);
impl_op_assign!(SubAssign, sub_assign);
impl_op_assign!(MulAssign, mul_assign);
impl_op_assign!(DivAssign, div_assign);
impl_op_assign!(RemAssign, rem_assign);

impl std::iter::Sum for FCT {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::default(), |a, b| a + b)
    }
}

impl<'a> std::iter::Sum<&'a FCT> for FCT {
    fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(Self::default(), |a, b| a + *b)
    }
}

macro_rules! impl_op_primitive {
    ($trait:ident, $method:ident, $($t:ty),*) => {
        $(
            impl std::ops::$trait<$t> for FCT {
                type Output = Self;
                fn $method(self, rhs: $t) -> Self::Output {
                    Self(self.0.$method(Decimal::from(rhs)))
                }
            }

            impl std::ops::$trait<FCT> for $t {
                type Output = FCT;
                fn $method(self, rhs: FCT) -> Self::Output {
                    FCT(Decimal::from(self).$method(rhs.0))
                }
            }
        )*
    };
}

impl_op_primitive!(Add, add, i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);
impl_op_primitive!(Sub, sub, i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);
impl_op_primitive!(Mul, mul, i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);
impl_op_primitive!(Div, div, i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);
impl_op_primitive!(Rem, rem, i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);

macro_rules! impl_op_float {
    ($trait:ident, $method:ident, $($t:ty, $from:ident),*) => {
        $(
            impl std::ops::$trait<$t> for FCT {
                type Output = Self;
                fn $method(self, rhs: $t) -> Self::Output {
                    Self(self.0.$method(Decimal::$from(rhs).unwrap_or_default()))
                }
            }

            impl std::ops::$trait<FCT> for $t {
                type Output = FCT;
                fn $method(self, rhs: FCT) -> Self::Output {
                    FCT(Decimal::$from(self).unwrap_or_default().$method(rhs.0))
                }
            }
        )*
    };
}

impl_op_float!(Add, add, f32, from_f32, f64, from_f64);
impl_op_float!(Sub, sub, f32, from_f32, f64, from_f64);
impl_op_float!(Mul, mul, f32, from_f32, f64, from_f64);
impl_op_float!(Div, div, f32, from_f32, f64, from_f64);
impl_op_float!(Rem, rem, f32, from_f32, f64, from_f64);
