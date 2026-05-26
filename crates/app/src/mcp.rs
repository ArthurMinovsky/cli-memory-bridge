use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::Utc;
use cli_memory_core::{ProviderKind, models::MessageRole};
use cli_memory_engine::{Embedder, RetrievalService, Storage};
use rmcp::{
    ErrorData,
    ServerHandler,
    ServiceExt,
    handler::server::wrapper::Parameters,
    schemars::{self, JsonSchema},
    tool,
    tool_handler,
    tool_router,
    transport::stdio,
};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::{
    bootstrap::{configured_db_path, configured_home, run_refresh},
    doctor,
};

pub fn health_check(status: &crate::doctor::DoctorStatus) -> Value {
    json!({
        "status": "ok",
        "db_ready": status.db_ready,
        "embedder_ready": status.embedder_ready,
        "vector_ready": status.vector_ready,
        "detected_providers": status.detected_providers,
    })
}

pub fn get_context_bundle(
    db_path: impl AsRef<Path>,
    query: &str,
    char_budget: usize,
) -> Result<Value> {
    let _ = run_refresh();
    let storage = Storage::open(db_path)?;
    let service = RetrievalService::from_storage(&storage)?;
    context_bundle_with_service(&service, query, char_budget)
}

pub fn context_bundle_with_service(
    service: &RetrievalService,
    query: &str,
    char_budget: usize,
) -> Result<Value> {
    let bundle = service.context_bundle(query, char_budget)?;
    Ok(json!({
        "status": "ok",
        "query": query,
        "char_budget": char_budget,
        "bundle": bundle,
    }))
}

pub fn context_bundle_with_hashing_service(
    db_path: impl AsRef<Path>,
    query: &str,
    char_budget: usize,
) -> Result<Value> {
    let storage = Storage::open(db_path)?;
    let service = RetrievalService::from_storage_with_embedder(&storage, Embedder::hashing(128))?;
    let bundle = service.context_bundle(query, char_budget)?;
    Ok(json!({
        "status": "ok",
        "query": query,
        "char_budget": char_budget,
        "bundle": bundle,
    }))
}

pub fn discover_providers() -> Result<Value> {
    let home = configured_home()?;
    let detected = cli_memory_integrations::detect_providers(&home)?;
    Ok(json!({
        "status": "ok",
        "providers": detected.into_iter().map(|item| {
            json!({
                "provider": item.provider.as_slug(),
                "paths": item.paths.into_iter().map(|path| path.display().to_string()).collect::<Vec<_>>(),
            })
        }).collect::<Vec<_>>(),
    }))
}

pub fn refresh_imports() -> Result<Value> {
    let summary = run_refresh()?;
    Ok(json!({
        "status": "ok",
        "provider_count": summary.provider_count,
        "checkpoint_count": summary.checkpoint_count,
        "imported_conversations": summary.imported_conversations,
        "imported_messages": summary.imported_messages,
        "providers": summary.providers,
    }))
}

pub fn resume_conversation(db_path: impl AsRef<Path>, hash_id: &str) -> Result<Value> {
    let _ = run_refresh();
    let storage = Storage::open(db_path)?;
    let bundle = storage.resume_bundle(hash_id)?;
    Ok(json!({
        "status": if bundle.is_some() { "ok" } else { "not_found" },
        "hash_id": hash_id,
        "bundle": bundle,
    }))
}

pub fn search_conversations(
    db_path: impl AsRef<Path>,
    query: &str,
    limit: usize,
) -> Result<Value> {
    let _ = run_refresh();
    let storage = Storage::open(db_path)?;
    let service = RetrievalService::from_storage(&storage)?;
    let results = service.search_lines(query, limit)?;
    Ok(json!({
        "status": "ok",
        "query": query,
        "results": results,
    }))
}

pub fn get_recent_history(db_path: impl AsRef<Path>, limit: usize) -> Result<Value> {
    let storage = Storage::open(db_path)?;
    let messages = storage.list_recent_messages(limit)?;
    Ok(json!({
        "status": "ok",
        "messages": messages.into_iter().map(|message| {
            json!({
                "provider": message.provider.as_slug(),
                "conversation_id": message.conversation_id,
                "message_id": message.message_id,
                "role": role_slug(message.role),
                "content": message.content,
            })
        }).collect::<Vec<_>>(),
    }))
}

pub fn search_history(db_path: impl AsRef<Path>, query: &str, limit: usize) -> Result<Value> {
    let storage = Storage::open(db_path)?;
    let results = storage.search_conversations(query, limit)?;
    Ok(json!({
        "status": "ok",
        "query": query,
        "results": results,
    }))
}

pub fn save_message(
    db_path: impl AsRef<Path>,
    provider: ProviderKind,
    conversation_id: &str,
    message_id: &str,
    role: MessageRole,
    content: &str,
) -> Result<Value> {
    let storage = Storage::open(db_path)?;
    storage.save_message(provider, conversation_id, message_id, role, content)?;
    let embedder = Embedder::model2vec_default().unwrap_or_else(|_| Embedder::hashing(128));
    let vector = embedder.embed_documents(&[content.to_owned()])?;
    storage.save_message_embedding(provider, conversation_id, message_id, &vector[0])?;
    Ok(json!({
        "status": "ok",
        "provider": provider.as_slug(),
        "conversation_id": conversation_id,
        "message_id": message_id,
    }))
}

pub fn save_conversation_turn(
    db_path: impl AsRef<Path>,
    provider: ProviderKind,
    conversation_id: &str,
    user_message: &str,
    assistant_message: &str,
) -> Result<Value> {
    let millis = Utc::now().timestamp_millis();
    let user_id = format!("user-{millis}");
    let assistant_id = format!("assistant-{millis}");
    save_message(
        &db_path,
        provider,
        conversation_id,
        &user_id,
        MessageRole::User,
        user_message,
    )?;
    save_message(
        &db_path,
        provider,
        conversation_id,
        &assistant_id,
        MessageRole::Assistant,
        assistant_message,
    )?;
    Ok(json!({
        "status": "ok",
        "provider": provider.as_slug(),
        "conversation_id": conversation_id,
        "message_ids": [user_id, assistant_id],
    }))
}

pub fn forget_conversation(
    db_path: impl AsRef<Path>,
    provider: ProviderKind,
    hash_id: &str,
) -> Result<Value> {
    let storage = Storage::open(db_path)?;
    let forgotten = storage.forget_conversation(provider, hash_id)?;
    Ok(json!({
        "status": if forgotten { "ok" } else { "not_found" },
        "provider": provider.as_slug(),
        "hash_id": hash_id,
        "forgotten": forgotten,
    }))
}

pub fn delete_history(
    db_path: impl AsRef<Path>,
    provider: ProviderKind,
    hash_id: &str,
) -> Result<Value> {
    let storage = Storage::open(db_path)?;
    let deleted = storage.delete_history(provider, hash_id)?;
    Ok(json!({
        "status": if deleted { "ok" } else { "not_found" },
        "provider": provider.as_slug(),
        "hash_id": hash_id,
        "deleted": deleted,
    }))
}

pub fn clear_session(
    db_path: impl AsRef<Path>,
    provider: ProviderKind,
    hash_id: &str,
) -> Result<Value> {
    let storage = Storage::open(db_path)?;
    let cleared = storage.clear_session(provider, hash_id)?;
    Ok(json!({
        "status": if cleared { "ok" } else { "not_found" },
        "provider": provider.as_slug(),
        "hash_id": hash_id,
        "cleared": cleared,
    }))
}

pub fn memory_stats(db_path: impl AsRef<Path>) -> Result<Value> {
    let storage = Storage::open(db_path)?;
    Ok(json!({
        "status": "ok",
        "active_conversations": storage.count_active_conversations()?,
        "total_conversations": storage.count_conversations()?,
        "total_messages": storage.count_messages()?,
        "total_embeddings": storage.count_message_embeddings()?,
        "total_checkpoints": storage.list_checkpoints()?.len(),
    }))
}

#[derive(Clone)]
struct CliMemoryMcpServer {
    db_path: PathBuf,
}

impl CliMemoryMcpServer {
    fn new() -> Result<Self> {
        Ok(Self {
            db_path: configured_db_path()?,
        })
    }
}

#[tool_router]
impl CliMemoryMcpServer {
    #[tool(name = "doctor", description = "Return cli-memory server and storage health.")]
    fn health_check(&self) -> std::result::Result<String, ErrorData> {
        render_json(health_check(&doctor::inspect().map_err(internal_error)?))
    }

    #[tool(name = "discover-providers", description = "Detect supported local transcript providers on this machine.")]
    fn discover_providers(&self) -> std::result::Result<String, ErrorData> {
        render_json(discover_providers().map_err(internal_error)?)
    }

    #[tool(name = "refresh", description = "Incrementally import newly changed conversation sources.")]
    fn refresh_imports(&self) -> std::result::Result<String, ErrorData> {
        render_json(refresh_imports().map_err(internal_error)?)
    }

    #[tool(name = "resume", description = "Resume a stored conversation transcript by stable hash id.")]
    fn resume_conversation(
        &self,
        Parameters(ResumeConversationArgs { hash_id }): Parameters<ResumeConversationArgs>,
    ) -> std::result::Result<String, ErrorData> {
        render_json(resume_conversation(&self.db_path, &hash_id).map_err(internal_error)?)
    }

    #[tool(name = "conv-search", description = "Search imported conversation content.")]
    fn search_conversations(
        &self,
        Parameters(SearchConversationsArgs { query, limit }): Parameters<SearchConversationsArgs>,
    ) -> std::result::Result<String, ErrorData> {
        render_json(
            search_conversations(&self.db_path, &query, limit.unwrap_or(10)).map_err(internal_error)?,
        )
    }

    #[tool(name = "context-bundle", description = "Build a retrieval bundle for a query.")]
    fn get_context_bundle(
        &self,
        Parameters(GetContextBundleArgs { query, char_budget }): Parameters<GetContextBundleArgs>,
    ) -> std::result::Result<String, ErrorData> {
        render_json(
            get_context_bundle(&self.db_path, &query, char_budget.unwrap_or(1200))
                .map_err(internal_error)?,
        )
    }

    #[tool(name = "recent-history", description = "Return recent stored messages.")]
    fn get_recent_history(
        &self,
        Parameters(GetRecentHistoryArgs { limit }): Parameters<GetRecentHistoryArgs>,
    ) -> std::result::Result<String, ErrorData> {
        render_json(get_recent_history(&self.db_path, limit.unwrap_or(20)).map_err(internal_error)?)
    }

    #[tool(name = "search-history", description = "Run storage-backed conversation search.")]
    fn search_history(
        &self,
        Parameters(SearchHistoryArgs { query, limit }): Parameters<SearchHistoryArgs>,
    ) -> std::result::Result<String, ErrorData> {
        render_json(search_history(&self.db_path, &query, limit.unwrap_or(10)).map_err(internal_error)?)
    }

    #[tool(name = "save-message", description = "Save a single provider-scoped message into memory.")]
    fn save_message(
        &self,
        Parameters(args): Parameters<SaveMessageArgs>,
    ) -> std::result::Result<String, ErrorData> {
        render_json(
            save_message(
                &self.db_path,
                parse_provider(&args.provider).map_err(invalid_params)?,
                &args.conversation_id,
                &args.message_id,
                parse_role(&args.role).map_err(invalid_params)?,
                &args.content,
            )
            .map_err(internal_error)?,
        )
    }

    #[tool(name = "save-conversation-turn", description = "Save a user/assistant turn pair.")]
    fn save_conversation_turn(
        &self,
        Parameters(args): Parameters<SaveConversationTurnArgs>,
    ) -> std::result::Result<String, ErrorData> {
        render_json(
            save_conversation_turn(
                &self.db_path,
                parse_provider(&args.provider).map_err(invalid_params)?,
                &args.conversation_id,
                &args.user_message,
                &args.assistant_message,
            )
            .map_err(internal_error)?,
        )
    }

    #[tool(name = "forget", description = "Soft-ban a conversation from future retrieval.")]
    fn forget_conversation(
        &self,
        Parameters(ProviderHashArgs { provider, hash_id }): Parameters<ProviderHashArgs>,
    ) -> std::result::Result<String, ErrorData> {
        render_json(
            forget_conversation(
                &self.db_path,
                parse_provider(&provider).map_err(invalid_params)?,
                &hash_id,
            )
            .map_err(internal_error)?,
        )
    }

    #[tool(name = "delete-history", description = "Delete a provider-scoped conversation from local storage.")]
    fn delete_history(
        &self,
        Parameters(ProviderHashArgs { provider, hash_id }): Parameters<ProviderHashArgs>,
    ) -> std::result::Result<String, ErrorData> {
        render_json(
            delete_history(
                &self.db_path,
                parse_provider(&provider).map_err(invalid_params)?,
                &hash_id,
            )
            .map_err(internal_error)?,
        )
    }

    #[tool(name = "clear-session", description = "Clear a provider-scoped session transcript while keeping indexes coherent.")]
    fn clear_session(
        &self,
        Parameters(ProviderHashArgs { provider, hash_id }): Parameters<ProviderHashArgs>,
    ) -> std::result::Result<String, ErrorData> {
        render_json(
            clear_session(
                &self.db_path,
                parse_provider(&provider).map_err(invalid_params)?,
                &hash_id,
            )
            .map_err(internal_error)?,
        )
    }

    #[tool(name = "stats", description = "Return memory database counts and embedding totals.")]
    fn memory_stats(&self) -> std::result::Result<String, ErrorData> {
        render_json(memory_stats(&self.db_path).map_err(internal_error)?)
    }
}

#[tool_handler(
    name = "cli-memory",
    version = "0.1.7",
    instructions = "Local cross-CLI memory retrieval server."
)]
impl ServerHandler for CliMemoryMcpServer {}

pub fn serve_stdio() -> Result<()> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("failed to build tokio runtime for MCP server")?;

    runtime.block_on(async {
        let server = CliMemoryMcpServer::new()?;
        let service = server.serve(stdio()).await?;
        service.waiting().await?;
        Ok(())
    })
}

fn render_json(value: Value) -> std::result::Result<String, ErrorData> {
    serde_json::to_string_pretty(&value)
        .map_err(|error| ErrorData::internal_error(error.to_string(), None))
}

fn internal_error(error: anyhow::Error) -> ErrorData {
    ErrorData::internal_error(error.to_string(), None)
}

fn invalid_params(error: anyhow::Error) -> ErrorData {
    ErrorData::invalid_params(error.to_string(), None)
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ResumeConversationArgs {
    hash_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct SearchConversationsArgs {
    query: String,
    limit: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct GetContextBundleArgs {
    query: String,
    char_budget: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct GetRecentHistoryArgs {
    limit: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct SearchHistoryArgs {
    query: String,
    limit: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct SaveMessageArgs {
    provider: String,
    conversation_id: String,
    message_id: String,
    role: String,
    content: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct SaveConversationTurnArgs {
    provider: String,
    conversation_id: String,
    user_message: String,
    assistant_message: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ProviderHashArgs {
    provider: String,
    hash_id: String,
}

fn parse_provider(value: &str) -> Result<ProviderKind> {
    Ok(ProviderKind::from_slug(value)?)
}

fn parse_role(value: &str) -> Result<MessageRole> {
    match value {
        "system" => Ok(MessageRole::System),
        "user" => Ok(MessageRole::User),
        "assistant" => Ok(MessageRole::Assistant),
        "tool" => Ok(MessageRole::Tool),
        _ => anyhow::bail!("unknown message role: {value}"),
    }
}

fn role_slug(role: MessageRole) -> &'static str {
    match role {
        MessageRole::System => "system",
        MessageRole::User => "user",
        MessageRole::Assistant => "assistant",
        MessageRole::Tool => "tool",
    }
}
