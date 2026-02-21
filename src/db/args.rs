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

pub struct Opt {
    pub end_query: Option<String>,
}

pub struct PgArgs {
    pub(crate) inner: PgArguments,
    pub(crate) opt: Option<Opt>,
}

impl Default for PgArgs {
    fn default() -> Self {
        Self::new()
    }
}

impl PgArgs {
    pub fn new() -> Self {
        Self { inner: PgArguments::default(), opt: None }
    }
}

pub trait PgArg {
    fn add_to(self, args: &mut PgArgs);
}

impl<T> PgArg for T
where
    T: for<'q> sqlx::Encode<'q, sqlx::Postgres> + sqlx::Type<sqlx::Postgres> + Send,
{
    fn add_to(self, args: &mut PgArgs) {
        use sqlx::Arguments;
        let _ = args.inner.add(self);
    }
}

impl PgArg for Opt {
    fn add_to(self, args: &mut PgArgs) {
        args.opt = Some(self);
    }
}

pub fn args_opt(end_query: &str) -> Opt {
    Opt { end_query: Some(end_query.to_string()) }
}

#[macro_export]
macro_rules! db_args {
    ($($x:expr),*) => {
        {
            let mut args = $crate::db::PgArgs::new();
            $(
                $crate::db::PgArg::add_to($x, &mut args);
            )*
            args
        }
    };
}

pub use db_args as args;
