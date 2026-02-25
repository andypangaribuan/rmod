/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

pub mod config;
pub mod conv;
pub mod fuse;
pub mod future;
pub mod http;
pub mod job;
pub mod store;
pub mod time;
pub mod uid;
pub mod util;
pub use fuse::fuse_handler;
pub mod db;
pub mod fct;
pub mod types;
pub use fct::FCT;
pub use sqlx;
pub use sqlx::{decode, postgres};
pub use types::ArcX;

// Re-exports for sqlx macros when using rmod as a sqlx proxy
pub use sqlx::{ColumnIndex, Decode, Encode, Error, FromRow, Row, Type};
pub type Result<T, E = sqlx::Error> = std::result::Result<T, E>;
