#![deny(unsafe_code)]

//! # Microgen Rust SDK
//!
//! Unofficial Rust client for [Microgen](https://microgen.id) – a no-code backend API.
//!
//! ## Quick start
//!
//! ```rust,no_run
//! use microgen_v3_sdk_rust::{MicrogenClient, MicrogenClientOptions};
//!
//! # async fn example() {
//! let mg = MicrogenClient::new(MicrogenClientOptions::new("my-api-key"));
//!
//! // ── Auth ──
//! let token_resp = mg
//!     .auth
//!     .login::<serde_json::Value>(&serde_json::json!({
//!         "email": "user@example.com",
//!         "password": "secret",
//!     }))
//!     .await
//!     .unwrap();
//! println!("token: {}", token_resp.token);
//!
//! // ── Database ──
//! let posts = mg.service("posts");
//!
//! let found = posts
//!     .find::<serde_json::Value>(None, None)
//!     .await
//!     .unwrap();
//! println!("{:#?}", found.data);
//!
//! // ── Storage ──
//! let file = mg.storage.upload(b"hello world".to_vec(), "hello.txt", None).await.unwrap();
//! println!("url: {}", file.url);
//! # }
//! ```

mod auth;
mod client;
mod error;
mod field;
mod query;
mod realtime;
mod storage;
pub mod types;

pub use auth::AuthClient;
pub use client::MicrogenClient;
pub use error::{MicrogenError, Result};
pub use field::{FieldClient, FieldResponse, FieldSingleResponse};
pub use query::QueryClient;
pub use realtime::RealtimeClient;
pub use storage::StorageClient;
pub use types::*;
