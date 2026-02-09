/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use sqlx::postgres::PgArguments;

pub struct PgArgs {
    pub(crate) inner: PgArguments,
}

impl Default for PgArgs {
    fn default() -> Self {
        Self::new()
    }
}

impl PgArgs {
    pub fn new() -> Self {
        Self { inner: PgArguments::default() }
    }

    pub fn push<T>(&mut self, val: T) -> &mut Self
    where
        T: for<'q> sqlx::Encode<'q, sqlx::Postgres> + sqlx::Type<sqlx::Postgres> + Send,
    {
        use sqlx::Arguments;
        let _ = self.inner.add(val);
        self
    }
}

#[macro_export]
macro_rules! db_args {
    ($($x:expr),*) => {
        {
            let mut args = $crate::db::PgArgs::new();
            $(
                args.push($x);
            )*
            args
        }
    };
}

pub use db_args as args;
