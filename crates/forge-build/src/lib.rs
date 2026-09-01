//! forge-build: build pipeline for Forge Engine.
//!
//! Takes a game project + target platform → produces a packaged, tested,
//! signed build artifact.

pub mod artifact;
pub mod cache;
pub mod parallel;
pub mod pipeline;

pub use cache::{BuildCache, CacheEntry};
pub use parallel::{build_parallel, BuildTarget, ParallelBuildRequest, ParallelBuildResult};
