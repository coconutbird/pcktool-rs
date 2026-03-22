//! # pcktool
//!
//! `#![no_std]` zero-copy library for parsing and writing Wwise PCK and BNK audio packages.
//!
//! Designed for modding Halo Wars: Definitive Edition audio files.
//!
//! ## Supported Formats
//!
//! - **PCK** (`.pck`) — Wwise package container holding soundbanks, streaming files, and external files
//! - **BNK** (`.bnk`) — Wwise soundbank containing HIRC hierarchy, embedded media, and metadata
//!
//! ## `no_std`
//!
//! This crate is `#![no_std]` and requires only `alloc`. All parsing operates on `&[u8]` slices
//! and all serialization writes to `Vec<u8>`. File I/O is the caller's responsibility.

#![no_std]
extern crate alloc;

pub mod bnk;
pub mod error;
pub mod hash;
pub mod pck;

pub use error::{Error, Result};
