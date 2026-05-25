use anyhow::{Result, ensure};
use turbovec::IdMapIndex;

#[derive(Debug, Clone, PartialEq)]
pub struct VectorHit {
    pub id: i64,
    /// Higher scores are more similar.
    pub score: f32,
}

pub struct VectorStore {
    dim: usize,
    backend: Backend,
}

enum Backend {
    Exact(Vec<(u64, Vec<f32>)>),
    Turbo(IdMapIndex),
}

impl VectorStore {
    pub fn new(dim: usize) -> Result<Self> {
        ensure!(dim > 0, "vector dimension must be greater than zero");
        let backend = if dim % 8 == 0 {
            Backend::Turbo(IdMapIndex::new(dim, 4))
        } else {
            Backend::Exact(Vec::new())
        };
        Ok(Self {
            dim,
            backend,
        })
    }

    /// Adds or replaces a vector for the given document id.
    pub fn add(&mut self, id: i64, vector: &[f32]) -> Result<()> {
        ensure!(
            vector.len() == self.dim,
            "vector dimension mismatch: expected {}, got {}",
            self.dim,
            vector.len()
        );

        let id = u64::try_from(id).map_err(|_| anyhow::anyhow!("vector id must be non-negative"))?;
        let vector = normalize(vector);

        match &mut self.backend {
            Backend::Exact(docs) => {
                if let Some((_, existing_vector)) = docs
                    .iter_mut()
                    .find(|(existing_id, _)| *existing_id == id)
                {
                    *existing_vector = vector;
                } else {
                    docs.push((id, vector));
                }
            }
            Backend::Turbo(index) => {
                if index.contains(id) {
                    index.remove(id);
                }
                index.add_with_ids(&vector, &[id])?;
            }
        }

        Ok(())
    }

    /// Returns hits sorted from highest to lowest cosine similarity.
    pub fn search(&self, query: &[f32], k: usize) -> Result<Vec<VectorHit>> {
        ensure!(
            query.len() == self.dim,
            "query dimension mismatch: expected {}, got {}",
            self.dim,
            query.len()
        );

        let query = normalize(query);
        match &self.backend {
            Backend::Exact(docs) => {
                let mut hits = docs
                    .iter()
                    .map(|(id, doc)| {
                        let score = query.iter().zip(doc).map(|(a, b)| a * b).sum::<f32>();
                        VectorHit {
                            id: *id as i64,
                            score,
                        }
                    })
                    .collect::<Vec<_>>();

                hits.sort_by(|left, right| right.score.total_cmp(&left.score));
                hits.truncate(k);
                Ok(hits)
            }
            Backend::Turbo(index) => {
                let (scores, ids) = index.search(&query, k);
                Ok(scores
                    .into_iter()
                    .zip(ids)
                    .map(|(score, id)| VectorHit {
                        id: id as i64,
                        score,
                    })
                    .collect())
            }
        }
    }
}

fn normalize(vector: &[f32]) -> Vec<f32> {
    let mut normalized = vector.to_vec();
    let norm = normalized
        .iter()
        .map(|value| value * value)
        .sum::<f32>()
        .sqrt();

    if norm > 0.0 {
        for value in &mut normalized {
            *value /= norm;
        }
    }

    normalized
}
