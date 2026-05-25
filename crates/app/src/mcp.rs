use std::{
    io::{self, BufRead, Write},
    path::Path,
};

use anyhow::{Context, Result, anyhow};
use chrono::Utc;
use cli_memory_core::{ProviderKind, models::MessageRole};
use cli_memory_engine::{Embedder, RetrievalService, Storage};
use serde_json::{Value, json};

use crate::{
    bootstrap::{configured_db_path, configured_home, run_refresh},
    doctor,
};

pub fn health_check(status: &crate::doctor::DoctorStatus) -> serde_json::Value {
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
) -> Result<serde_json::Value> {
    let _ = run_refresh();
    let storage = Storage::open(db_path)?;
    let service = RetrievalService::from_storage(&storage)?;
    context_bundle_with_service(&service, query, char_budget)
}

pub fn context_bundle_with_service(
    service: &RetrievalService,
    query: &str,
    char_budget: usize,
) -> Result<serde_json::Value> {
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
) -> Result<serde_json::Value> {
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

pub fn discover_providers() -> Result<serde_json::Value> {
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

pub fn refresh_imports() -> Result<serde_json::Value> {
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

pub fn resume_conversation(db_path: impl AsRef<Path>, hash_id: &str) -> Result<serde_json::Value> {
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
) -> Result<serde_json::Value> {
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

pub fn get_recent_history(
    db_path: impl AsRef<Path>,
    limit: usize,
) -> Result<serde_json::Value> {
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

pub fn search_history(
    db_path: impl AsRef<Path>,
    query: &str,
    limit: usize,
) -> Result<serde_json::Value> {
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
) -> Result<serde_json::Value> {
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
) -> Result<serde_json::Value> {
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
) -> Result<serde_json::Value> {
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
) -> Result<serde_json::Value> {
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
) -> Result<serde_json::Value> {
    let storage = Storage::open(db_path)?;
    let cleared = storage.clear_session(provider, hash_id)?;
    Ok(json!({
        "status": if cleared { "ok" } else { "not_found" },
        "provider": provider.as_slug(),
        "hash_id": hash_id,
        "cleared": cleared,
    }))
}

pub fn memory_stats(db_path: impl AsRef<Path>) -> Result<serde_json::Value> {
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

pub fn serve_stdio() -> Result<()> {
    let manifest = json!({
        "status": "ok",
        "transport": "stdio-jsonl",
        "tools": [
            "health_check",
            "discover_providers",
            "refresh_imports",
            "resume_conversation",
            "search_conversations",
            "get_context_bundle",
            "get_recent_history",
            "search_history",
            "save_message",
            "save_conversation_turn",
            "forget_conversation",
            "delete_history",
            "clear_session",
            "memory_stats"
        ]
    });

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    writeln!(stdout, "{}", serde_json::to_string(&manifest)?)?;

    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let response = handle_stdio_request(&line)
            .unwrap_or_else(|error| json!({ "status": "error", "error": error.to_string() }));
        writeln!(stdout, "{}", serde_json::to_string(&response)?)?;
        stdout.flush()?;
    }

    Ok(())
}

pub fn handle_stdio_request(line: &str) -> Result<serde_json::Value> {
    let request: Value = serde_json::from_str(line).context("invalid JSON request")?;
    let tool = request
        .get("tool")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("request is missing tool"))?;
    let args = request.get("args").cloned().unwrap_or_else(|| json!({}));
    let db_path = configured_db_path()?;

    match tool {
        "health_check" => Ok(health_check(&doctor::inspect()?)),
        "discover_providers" => discover_providers(),
        "refresh_imports" => refresh_imports(),
        "resume_conversation" => resume_conversation(
            &db_path,
            required_string(&args, "hash_id")?,
        ),
        "search_conversations" => search_conversations(
            &db_path,
            required_string(&args, "query")?,
            optional_usize(&args, "limit", 10),
        ),
        "get_context_bundle" => get_context_bundle(
            &db_path,
            required_string(&args, "query")?,
            optional_usize(&args, "char_budget", 1200),
        ),
        "get_recent_history" => get_recent_history(&db_path, optional_usize(&args, "limit", 20)),
        "search_history" => search_history(
            &db_path,
            required_string(&args, "query")?,
            optional_usize(&args, "limit", 10),
        ),
        "save_message" => save_message(
            &db_path,
            parse_provider(required_string(&args, "provider")?)?,
            required_string(&args, "conversation_id")?,
            required_string(&args, "message_id")?,
            parse_role(required_string(&args, "role")?)?,
            required_string(&args, "content")?,
        ),
        "save_conversation_turn" => save_conversation_turn(
            &db_path,
            parse_provider(required_string(&args, "provider")?)?,
            required_string(&args, "conversation_id")?,
            required_string(&args, "user_message")?,
            required_string(&args, "assistant_message")?,
        ),
        "forget_conversation" => forget_conversation(
            &db_path,
            parse_provider(required_string(&args, "provider")?)?,
            required_string(&args, "hash_id")?,
        ),
        "delete_history" => delete_history(
            &db_path,
            parse_provider(required_string(&args, "provider")?)?,
            required_string(&args, "hash_id")?,
        ),
        "clear_session" => clear_session(
            &db_path,
            parse_provider(required_string(&args, "provider")?)?,
            required_string(&args, "hash_id")?,
        ),
        "memory_stats" => memory_stats(&db_path),
        _ => Err(anyhow!("unknown tool: {tool}")),
    }
}

fn required_string<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("missing required string argument: {key}"))
}

fn optional_usize(value: &Value, key: &str, default: usize) -> usize {
    value
        .get(key)
        .and_then(Value::as_u64)
        .and_then(|raw| usize::try_from(raw).ok())
        .unwrap_or(default)
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
        _ => Err(anyhow!("unknown message role: {value}")),
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
