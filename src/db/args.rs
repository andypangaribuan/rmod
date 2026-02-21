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
#[path = "test/args.rs"]
mod tests;

use sqlx::postgres::PgArguments;

pub type PgArgCollector = Box<dyn Fn(&mut PgArguments) + Send + Sync>;
pub type OptValidator<T> = Box<dyn Fn(&Option<T>) -> bool + Send + Sync>;
pub type OptValidatorAll<T> = Box<dyn Fn(&Vec<T>) -> bool + Send + Sync>;

pub struct Opt<T = ()> {
    pub end_query: Option<String>,
    pub force_rw: Option<bool>,
    pub validate: Option<OptValidator<T>>,
    pub validate_all: Option<OptValidatorAll<T>>,
}

impl<T> Default for Opt<T> {
    fn default() -> Self {
        Self { end_query: None, force_rw: None, validate: None, validate_all: None }
    }
}

pub struct PgArgs<T = ()> {
    pub(crate) collectors: Vec<PgArgCollector>,
    pub(crate) opt: Option<Opt<T>>,
}

impl<T> Default for PgArgs<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> PgArgs<T> {
    pub fn new() -> Self {
        Self { collectors: Vec::new(), opt: None }
    }

    pub(crate) fn build_inner(&self) -> PgArguments {
        let mut inner = PgArguments::default();
        for collector in &self.collectors {
            collector(&mut inner);
        }
        inner
    }

    pub(crate) fn is_force_rw(&self) -> bool {
        self.opt.as_ref().and_then(|o| o.force_rw).unwrap_or(false)
    }
}

pub trait PgArg<T> {
    fn add_to(self, args: &mut PgArgs<T>);
}

impl<T, V> PgArg<T> for V
where
    V: for<'q> sqlx::Encode<'q, sqlx::Postgres> + sqlx::Type<sqlx::Postgres> + Send + Sync + Clone + 'static,
{
    fn add_to(self, args: &mut PgArgs<T>) {
        args.collectors.push(Box::new(move |inner| {
            use sqlx::Arguments;
            let _ = inner.add(self.clone());
        }));
    }
}

// I need to be careful with Clone on T. Actually, the Encode types usually implement Clone or are small.
// But some might not. Let's assume they are Cloneable for now or use a wrapper if needed.
// Wait, sqlx::Encode doesn't imply Clone. But primitive types do.
// Let's refine the trait impl to handle Clone.

impl<T> PgArg<T> for Opt<T> {
    fn add_to(self, args: &mut PgArgs<T>) {
        args.opt = Some(self);
    }
}

pub fn args_opt<T>(end_query: &str) -> Opt<T> {
    Opt { end_query: Some(end_query.to_string()), force_rw: None, validate: None, validate_all: None }
}

pub fn args_opt_rw<T>(end_query: &str) -> Opt<T> {
    Opt { end_query: Some(end_query.to_string()), force_rw: Some(true), validate: None, validate_all: None }
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
