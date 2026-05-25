use std::collections::BTreeMap;

use anyhow::Result;
use cli_memory_core::models::MessageRole;

use crate::{Embedder, EmbeddedMessageRow, MessageRow, Storage, VectorStore};

#[derive(Clone, Debug, PartialEq, Eq)]
struct RetrievalDocument {
    provider: String,
    conversation_id: String,
    message_id: String,
    role: MessageRole,
    content: String,
}

pub struct RetrievalService {
    embedder: Embedder,
    index: VectorStore,
    docs: BTreeMap<i64, RetrievalDocument>,
    next_id: i64,
}

impl RetrievalService {
    pub fn hashing(dim: usize) -> Result<Self> {
        let embedder = Embedder::hashing(dim);
        let index = VectorStore::new(embedder.dimension())?;
        Ok(Self {
            embedder,
            index,
            docs: BTreeMap::new(),
            next_id: 1,
        })
    }

    pub fn production() -> Result<Self> {
        let embedder = Embedder::model2vec_default().unwrap_or_else(|_| Embedder::hashing(128));
        let index = VectorStore::new(embedder.dimension())?;
        Ok(Self {
            embedder,
            index,
            docs: BTreeMap::new(),
            next_id: 1,
        })
    }

    pub fn from_storage(storage: &Storage) -> Result<Self> {
        let embedder = Embedder::model2vec_default().unwrap_or_else(|_| Embedder::hashing(128));
        Self::from_storage_with_embedder(storage, embedder)
    }

    pub fn from_storage_with_embedder(storage: &Storage, embedder: Embedder) -> Result<Self> {
        let mut service = Self {
            index: VectorStore::new(embedder.dimension())?,
            embedder,
            docs: BTreeMap::new(),
            next_id: 1,
        };
        let embedded = storage.list_embedded_messages()?;
        if !embedded.is_empty() {
            for message in embedded {
                service.ingest_embedded_message(message)?;
            }
            return Ok(service);
        }

        let messages = storage.list_messages()?;
        if !messages.is_empty() {
            service.ingest_messages_batched(messages)?;
        }
        Ok(service)
    }

    pub fn ingest_text(
        &mut self,
        provider: &str,
        conversation_id: &str,
        text: &str,
    ) -> Result<()> {
        self.ingest_document(RetrievalDocument {
            provider: provider.to_owned(),
            conversation_id: conversation_id.to_owned(),
            message_id: format!("generated-{}", self.next_id),
            role: MessageRole::Assistant,
            content: text.to_owned(),
        })
    }

    pub fn search_lines(&self, query: &str, limit: usize) -> Result<Vec<String>> {
        let vector = self.embedder.embed_query(&[query.to_owned()])?;
        let hits = self.index.search(&vector[0], limit)?;

        let mut out = Vec::new();
        for hit in hits {
            let Some(doc) = self.docs.get(&hit.id) else {
                continue;
            };
            out.push(format!(
                "[{}:{}] {}",
                doc.provider, doc.conversation_id, doc.content
            ));
        }

        Ok(out)
    }

    pub fn context_bundle(&self, query: &str, char_budget: usize) -> Result<String> {
        let lines = self.search_lines(query, self.docs.len())?;
        let mut out = String::new();
        for line in lines {
            let separator = if out.is_empty() { "" } else { "\n\n" };
            if out.len() + separator.len() + line.len() > char_budget {
                break;
            }

            out.push_str(separator);
            out.push_str(&line);
        }

        Ok(out)
    }

    fn ingest_embedded_message(&mut self, message: EmbeddedMessageRow) -> Result<()> {
        let id = self.next_id;
        self.next_id += 1;
        self.index.add(id, &message.embedding)?;
        self.docs.insert(
            id,
            RetrievalDocument {
                provider: message.provider.as_slug().to_owned(),
                conversation_id: message.conversation_id,
                message_id: message.message_id,
                role: message.role,
                content: message.content,
            },
        );
        Ok(())
    }

    fn ingest_messages_batched(&mut self, messages: Vec<MessageRow>) -> Result<()> {
        let texts = messages
            .iter()
            .map(|message| message.content.clone())
            .collect::<Vec<_>>();
        let vectors = self.embedder.embed_documents(&texts)?;

        for (message, vector) in messages.into_iter().zip(vectors.into_iter()) {
            let id = self.next_id;
            self.next_id += 1;
            self.index.add(id, &vector)?;
            self.docs.insert(
                id,
                RetrievalDocument {
                    provider: message.provider.as_slug().to_owned(),
                    conversation_id: message.conversation_id,
                    message_id: message.message_id,
                    role: message.role,
                    content: message.content,
                },
            );
        }

        Ok(())
    }

    fn ingest_document(&mut self, doc: RetrievalDocument) -> Result<()> {
        let id = self.next_id;
        self.next_id += 1;

        let vector = self.embedder.embed_documents(&[doc.content.clone()])?;
        self.index.add(id, &vector[0])?;
        self.docs.insert(id, doc);
        Ok(())
    }
}

pub fn test_service() -> Result<RetrievalService> {
    RetrievalService::hashing(128)
}
