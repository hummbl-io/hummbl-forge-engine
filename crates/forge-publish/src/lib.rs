//! forge-publish: publish pipeline for Forge Engine.
//!
//! Takes a build artifact + target store → produces a store-ready package
//! and uploads it (or dry-runs the packaging).

pub mod direct;
pub mod linux_native;
pub mod store;
