use std::{
    collections::BTreeMap,
    env,
    fs,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use anyhow::{Context, Result};
use chrono::Utc;
use cli_memory_core::{ProviderKind, models::ConversationTranscript};
use cli_memory_engine::{Checkpoint, Embedder, Storage};
use cli_memory_integrations::{
    DetectedProvider, detect_providers, import_antigravity_cli, import_claude, import_codex,
    import_copilot, import_gemini, import_hermes, import_opencode, import_zed,
};

pub struct BootstrapSummary {
    pub provider_count: usize,
    pub checkpoint_count: usize,
    pub imported_conversations: usize,
    pub imported_messages: usize,
    pub providers: Vec<String>,
}

pub fn run_init() -> Result<BootstrapSummary> {
    run_sync()
}

pub fn run_refresh() -> Result<BootstrapSummary> {
    run_sync()
}

struct PendingImport {
    source_path: String,
    fingerprint: String,
    conversation: ConversationTranscript,
}

fn run_sync() -> Result<BootstrapSummary> {
    let home = configured_home()?;
    let detected = detect_providers(&home)?;
    let db_path = configured_data_dir()?.join("db.sqlite3");
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create data directory {}", parent.display()))?;
    }

    let storage = Storage::open(&db_path)?;
    storage.initialize()?;
    let existing_checkpoints = storage
        .list_checkpoints()?
        .into_iter()
        .map(|row| {
            (
                (row.provider.as_slug().to_owned(), row.source_path),
                row.fingerprint,
            )
        })
        .collect::<BTreeMap<_, _>>();
    let imported_sources = storage
        .conversation_states()?
        .into_iter()
        .map(|row| {
            (
                (row.provider.as_slug().to_owned(), row.source_path),
                row.source_fingerprint,
            )
        })
        .collect::<BTreeMap<_, _>>();

    let mut checkpoint_count = 0usize;
    let mut providers = Vec::new();
    let mut imported_conversations = 0usize;
    let mut imported_messages = 0usize;
    let mut pending_imports = Vec::new();
    for provider in &detected {
        providers.push(provider.provider.as_slug().to_owned());
        let importable_sources = resolve_importable_sources(provider)?;
        for path in &importable_sources {
            let source_path = path.display().to_string();
            let fingerprint = fingerprint_for_path(path)?;
            let source_key = (provider.provider.as_slug().to_owned(), source_path.clone());

            let already_imported = imported_sources
                .get(&source_key)
                .is_some_and(|known| known == &fingerprint);
            if existing_checkpoints
                .get(&source_key)
                .is_some_and(|known| known == &fingerprint)
                && already_imported
            {
                checkpoint_count += 1;
                continue;
            }

            let transcript = import_for_provider(provider.provider, path)?;
            let conversation = transcript.into_conversation();
            if !conversation.messages.is_empty() {
                if storage.is_conversation_forgotten(
                    conversation.locator.provider,
                    &conversation.locator.conversation_id,
                )? {
                    storage.save_checkpoint(&Checkpoint {
                        provider: provider.provider,
                        source_path,
                        fingerprint,
                        updated_at: Utc::now(),
                    })?;
                    checkpoint_count += 1;
                    continue;
                }

                if storage.conversation_exists(
                    conversation.locator.provider,
                    &conversation.locator.conversation_id,
                )? {
                    storage.save_conversation_state(
                        conversation.locator.provider,
                        &conversation.locator.conversation_id,
                        &source_path,
                        &fingerprint,
                    )?;
                } else {
                    pending_imports.push(PendingImport {
                        source_path: source_path.clone(),
                        fingerprint: fingerprint.clone(),
                        conversation,
                    });
                }
            }

            storage.save_checkpoint(&Checkpoint {
                provider: provider.provider,
                source_path,
                fingerprint,
                updated_at: Utc::now(),
            })?;
            checkpoint_count += 1;
        }
    }

    if !pending_imports.is_empty() {
        let embedder = Embedder::global();
        let texts = pending_imports
            .iter()
            .flat_map(|item| item.conversation.messages.iter().map(|message| message.content.clone()))
            .collect::<Vec<_>>();
        let embeddings = embedder.embed_documents(&texts)?;
        let mut offset = 0usize;

        for pending in pending_imports {
            let message_count = pending.conversation.messages.len();
            storage.save_transcript(&pending.conversation)?;
            storage.save_conversation_state(
                pending.conversation.locator.provider,
                &pending.conversation.locator.conversation_id,
                &pending.source_path,
                &pending.fingerprint,
            )?;
            for message in &pending.conversation.messages {
                let embedding = &embeddings[offset];
                storage.save_message_embedding(
                    pending.conversation.locator.provider,
                    &pending.conversation.locator.conversation_id,
                    &message.message_id,
                    embedding,
                )?;
                offset += 1;
            }
            imported_conversations += 1;
            imported_messages += message_count;
        }
    }

    Ok(BootstrapSummary {
        provider_count: detected.len(),
        checkpoint_count,
        imported_conversations,
        imported_messages,
        providers,
    })
}

fn resolve_importable_sources(provider: &DetectedProvider) -> Result<Vec<PathBuf>> {
    let mut sources = Vec::new();
    for path in &provider.paths {
        if path.is_file() {
            if matches_provider_file(provider.provider, path) {
                sources.push(path.clone());
            }
            continue;
        }

        if path.is_dir() {
            collect_importable_paths(provider.provider, path, &mut sources)?;
        }
    }

    sources.sort();
    sources.dedup();
    Ok(sources)
}

fn collect_importable_paths(
    provider: ProviderKind,
    dir: &Path,
    out: &mut Vec<PathBuf>,
) -> Result<()> {
    if provider == ProviderKind::AntigravityCli {
        let looks_like_brain_dir = dir
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.contains('-'))
            && (dir.join("task.md").exists()
                || dir.join("implementation_plan.md").exists()
                || dir.join("walkthrough.md").exists());
        if looks_like_brain_dir {
            out.push(dir.to_path_buf());
            return Ok(());
        }
    }

    for entry in fs::read_dir(dir)
        .with_context(|| format!("failed to read directory {}", dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_importable_paths(provider, &path, out)?;
        } else if matches_provider_file(provider, &path) {
            out.push(path);
        }
    }

    Ok(())
}

fn matches_provider_file(provider: ProviderKind, path: &Path) -> bool {
    let file_name = path.file_name().and_then(|name| name.to_str()).unwrap_or_default();
    let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or_default();

    match provider {
        ProviderKind::Claude | ProviderKind::Codex | ProviderKind::Copilot | ProviderKind::Hermes => {
            extension == "jsonl"
        }
        ProviderKind::Gemini => {
            file_name.starts_with("session-") && matches!(extension, "json" | "jsonl")
        }
        ProviderKind::Zed => file_name.ends_with(".zed.json"),
        ProviderKind::OpenCode => {
            (extension == "json" && file_name.starts_with("ses_")) || file_name == "opencode.db"
        }
        ProviderKind::AntigravityCli => false,
    }
}

fn import_for_provider(provider: ProviderKind, path: &Path) -> Result<cli_memory_integrations::ImportedTranscript> {
    match provider {
        ProviderKind::Claude => import_claude(path),
        ProviderKind::Codex => import_codex(path),
        ProviderKind::Gemini => import_gemini(path),
        ProviderKind::Copilot => import_copilot(path),
        ProviderKind::Zed => import_zed(path),
        ProviderKind::OpenCode => import_opencode(path),
        ProviderKind::Hermes => import_hermes(path),
        ProviderKind::AntigravityCli => import_antigravity_cli(path),
    }
}

pub fn configured_home() -> Result<PathBuf> {
    if let Some(path) = env::var_os("CLI_MEMORY_HOME") {
        return Ok(PathBuf::from(path));
    }

    env::var_os("HOME")
        .map(PathBuf::from)
        .context("HOME is not set and CLI_MEMORY_HOME was not provided")
}

pub fn configured_data_dir() -> Result<PathBuf> {
    if let Some(path) = env::var_os("CLI_MEMORY_DATA_DIR") {
        return Ok(PathBuf::from(path));
    }

    Ok(configured_home()?.join(".cli-memory-bridge-rs"))
}

pub fn configured_db_path() -> Result<PathBuf> {
    Ok(configured_data_dir()?.join("db.sqlite3"))
}

fn fingerprint_for_path(path: &Path) -> Result<String> {
    if path.is_dir() {
        let fingerprint = fingerprint_for_dir(path)?;
        return Ok(format!("dir:{fingerprint}"));
    }

    let metadata = fs::metadata(path)
        .with_context(|| format!("failed to read metadata for {}", path.display()))?;
    let modified = metadata
        .modified()
        .ok()
        .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
        .map(|value| value.as_secs())
        .unwrap_or(0);

    let kind = if metadata.is_dir() { "dir" } else { "file" };
    Ok(format!("{kind}:{}:{modified}", metadata.len()))
}

fn fingerprint_for_dir(path: &Path) -> Result<String> {
    let mut entries = Vec::new();
    collect_dir_fingerprint_entries(path, &mut entries)?;
    entries.sort();
    Ok(entries.join("|"))
}

fn collect_dir_fingerprint_entries(dir: &Path, out: &mut Vec<String>) -> Result<()> {
    for entry in fs::read_dir(dir)
        .with_context(|| format!("failed to read directory {}", dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_dir_fingerprint_entries(&path, out)?;
            continue;
        }

        let metadata = fs::metadata(&path)
            .with_context(|| format!("failed to read metadata for {}", path.display()))?;
        let modified = metadata
            .modified()
            .ok()
            .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
            .map(|value| value.as_secs())
            .unwrap_or(0);
        out.push(format!(
            "{}:{}:{}",
            path.display(),
            metadata.len(),
            modified
        ));
    }

    Ok(())
}
