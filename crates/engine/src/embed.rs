use anyhow::{Result, ensure};
use model2vec_rs::model::StaticModel;

use crate::model_cache::resolve_model_dir;

#[derive(Clone)]
pub struct Embedder {
    dim: usize,
    backend: Backend,
}

#[derive(Clone)]
enum Backend {
    Hashing,
    Model2Vec(std::sync::Arc<StaticModel>),
}

impl Embedder {
    pub fn hashing(dim: usize) -> Self {
        Self {
            dim,
            backend: Backend::Hashing,
        }
    }

    pub fn model2vec_default() -> Result<Self> {
        let model_dir = resolve_model_dir()?;
        let model = StaticModel::from_pretrained(&model_dir, None, None, None)?;
        let dim = model.encode_single("dimension probe").len();
        ensure!(
            dim > 0,
            "model2vec returned an empty embedding dimension"
        );

        Ok(Self {
            dim,
            backend: Backend::Model2Vec(std::sync::Arc::new(model)),
        })
    }

    pub fn global() -> Self {
        static GLOBAL_EMBEDDER: std::sync::OnceLock<Embedder> = std::sync::OnceLock::new();
        GLOBAL_EMBEDDER.get_or_init(|| {
            Self::model2vec_default().unwrap_or_else(|_| Self::hashing(128))
        }).clone()
    }

    pub fn dimension(&self) -> usize {
        self.dim
    }

    pub fn embed_query(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        self.embed_documents(texts)
    }

    pub fn embed_documents(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        ensure!(
            self.dim > 0,
            "embedding dimension must be greater than zero"
        );

        match self.backend {
            Backend::Hashing => Ok(texts
                .iter()
                .map(|text| embed_text(text, self.dim))
                .collect::<Vec<_>>()),
            Backend::Model2Vec(ref model) => Ok(model.encode(texts)),
        }
    }
}

fn embed_text(text: &str, dim: usize) -> Vec<f32> {
    let mut out = vec![0.0; dim];

    for token in text.split_whitespace().filter(|token| !token.is_empty()) {
        let index = stable_hash(token) % dim;
        out[index] += 1.0;
    }

    let norm = out.iter().map(|value| value * value).sum::<f32>().sqrt();
    if norm > 0.0 {
        for value in &mut out {
            *value /= norm;
        }
    }

    out
}

fn stable_hash(token: &str) -> usize {
    token.bytes().fold(0usize, |hash, byte| {
        hash.wrapping_mul(33)
            .wrapping_add(usize::from(byte.to_ascii_lowercase()))
    })
}
