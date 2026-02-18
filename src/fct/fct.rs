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

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FCT(pub Decimal);

impl FCT {
    pub fn new(val: Decimal) -> Self {
        Self(val)
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
