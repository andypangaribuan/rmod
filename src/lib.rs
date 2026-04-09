/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

pub mod cache;
pub mod config;
pub mod conv;
pub mod db;
pub mod fct;
pub mod fuse;
pub mod future;
pub mod http;
pub mod job;
pub mod lock;
pub mod store;
pub mod time;
pub mod types;
pub mod uid;
pub mod util;

pub use fct::FCT;
pub use fuse::fuse_handler;
pub use sqlx;
pub use sqlx::{decode, postgres};
pub use types::ArcX;

// Proxy re-exports for common dependencies
pub use ::serde;
pub use ::serde::{Deserialize, Serialize, de, ser};
pub use ::serde_json as json;
pub use axum;
pub use bytes;
pub use chrono;
pub use chrono::{DateTime, Datelike, FixedOffset, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Timelike, Utc};
pub use prost;
pub use prost::alloc;
#[doc(hidden)]
pub use prost::*;
pub use rust_decimal;
pub use rust_decimal::*;
pub use rust_decimal_macros;
pub use rust_decimal_macros::dec;
pub use rustls;
pub use tokio;
pub use tokio::main;
pub use tokio::runtime;
pub use tonic;
pub use tonic::async_trait;
pub use tonic::client;
pub use tonic::codegen;
pub use tonic::server;
pub use tonic::transport;
#[doc(hidden)]
pub use tonic::*;
pub use tonic_health;

// Re-exports for sqlx macros when using rmod as a sqlx proxy
pub use sqlx::{ColumnIndex, Decode, Encode, Error, FromRow, Row, Type};
pub type Result<T, E = sqlx::Error> = std::result::Result<T, E>;
