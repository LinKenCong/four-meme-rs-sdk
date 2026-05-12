//! Programmatic Rust SDK for Four.meme REST APIs and BSC contracts.

pub mod api;
pub mod client;
pub mod config;
pub mod contracts;
pub mod eip8004;
pub mod error;
pub mod events;
pub mod trade;
pub mod types;
pub mod utils;
pub mod wallet;

pub use client::FourMemeSdk;
pub use config::{Addresses, SdkConfig};
pub use error::{Result, SdkError};
