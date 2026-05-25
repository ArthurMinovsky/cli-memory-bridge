//! Engine library crate.

#[cfg(any(target_os = "macos", target_os = "linux"))]
extern crate blas_src;

mod checkpoints;
mod embed;
mod model_cache;
mod retrieval;
mod storage;
mod vector;

pub use checkpoints::Checkpoint;
pub use embed::Embedder;
pub use model_cache::{current_model_dir, model_cache_ready};
pub use retrieval::{RetrievalService, test_service};
pub use storage::{CheckpointRow, ConversationStateRow, EmbeddedMessageRow, MessageRow, Storage};
pub use vector::{VectorHit, VectorStore};
