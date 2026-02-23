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
    pub(crate) table_name: Option<String>,
    pub(crate) tail_query: Option<String>,
    pub(crate) force_rw: Option<bool>,
    pub(crate) with_deleted_at: Option<bool>,
    pub(crate) validate: Option<OptValidator<T>>,
    pub(crate) validate_all: Option<OptValidatorAll<T>>,
}

impl<T> Default for Opt<T> {
    fn default() -> Self {
        Self { table_name: None, tail_query: None, force_rw: None, with_deleted_at: None, validate: None, validate_all: None }
    }
}

impl<T> Opt<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn table_name(mut self, table_name: &str) -> Self {
        self.table_name = Some(table_name.to_string());
        self
    }

    pub fn tail_query(mut self, query: &str) -> Self {
        self.tail_query = Some(query.to_string());
        self
    }

    pub fn force_rw(mut self) -> Self {
        self.force_rw = Some(true);
        self
    }

    pub fn with_deleted_at(mut self, val: bool) -> Self {
        self.with_deleted_at = Some(val);
        self
    }

    pub fn validate(mut self, f: impl Fn(&Option<T>) -> bool + Send + Sync + 'static) -> Self {
        self.validate = Some(Box::new(f));
        self
    }

    pub fn validate_all(mut self, f: impl Fn(&Vec<T>) -> bool + Send + Sync + 'static) -> Self {
        self.validate_all = Some(Box::new(f));
        self
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

    pub fn take_opt(&mut self) -> Option<Opt<T>> {
        self.opt.take()
    }

    pub fn set_opt(&mut self, opt: Option<Opt<T>>) {
        self.opt = opt;
    }

    pub fn push<V: PgArg<T>>(&mut self, arg: V) -> &mut Self {
        arg.add_to(self);
        self
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

impl<T> PgArg<T> for Opt<T> {
    fn add_to(self, args: &mut PgArgs<T>) {
        args.opt = Some(self);
    }
}

impl<T> PgArg<T> for PgArgs<T> {
    fn add_to(mut self, args: &mut PgArgs<T>) {
        if !self.collectors.is_empty() {
            args.collectors.append(&mut self.collectors);
        }

        if let Some(opt) = self.take_opt() {
            args.set_opt(Some(opt));
        }
    }
}

pub fn args_opt<T>() -> Opt<T> {
    Opt { table_name: None, tail_query: None, force_rw: None, with_deleted_at: None, validate: None, validate_all: None }
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
