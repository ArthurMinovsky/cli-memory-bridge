use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use clap::Parser;
use cli_memory_app::bootstrap::{configured_db_path, run_init, run_refresh};
use cli_memory_app::cli::{Cli, Commands};
use cli_memory_app::{doctor, install, mcp};
use cli_memory_core::ProviderKind;
use cli_memory_engine::Storage;

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init => {
            let summary = run_init().expect("init should succeed");
            let binary_path = std::env::current_exe().expect("current executable path should resolve");
            let home = cli_memory_app::bootstrap::configured_home().expect("home should resolve");
            let detected = cli_memory_integrations::detect_providers(&home)
                .expect("provider detection should succeed after init");
            let install_summary = install::ensure_detected_integrations(
                &home,
                &detected,
                &binary_path.display().to_string(),
            )
            .expect("detected integrations should install");
            println!(
                "detected {} providers, checkpointed {} sources, imported {} conversations / {} messages",
                summary.provider_count, summary.checkpoint_count
                , summary.imported_conversations, summary.imported_messages
            );
            if !summary.providers.is_empty() {
                println!("{}", summary.providers.join(","));
            }
            if !install_summary.installed.is_empty() {
                println!(
                    "installed cli-memory integrations: {}",
                    install_summary.installed.join(",")
                );
            }
            if !install_summary.skipped.is_empty() {
                println!(
                    "skipped existing integrations: {}",
                    install_summary.skipped.join(",")
                );
            }
        }
        Commands::Refresh => {
            let summary = run_refresh().expect("refresh should succeed");
            println!(
                "refreshed {} providers, checkpointed {} sources, imported {} conversations / {} messages",
                summary.provider_count, summary.checkpoint_count
                , summary.imported_conversations, summary.imported_messages
            );
            if !summary.providers.is_empty() {
                println!("{}", summary.providers.join(","));
            }
        }
        Commands::Install { provider, all } => {
            let binary_path = std::env::current_exe().expect("current executable path should resolve");
            let bundle = if all {
                install::render_install_all_bundle(&binary_path.display().to_string())
                    .expect("install-all bundle should render")
            } else {
                let provider = provider
                    .as_deref()
                    .expect("provider should be passed unless --all is used");
                let provider = ProviderKind::from_slug(provider).expect("provider should be valid");
                install::render_install_bundle(
                    provider,
                    &binary_path.display().to_string(),
                )
                .expect("install bundle should render")
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&bundle).expect("install bundle should serialize")
            );
        }
        Commands::Unlink { provider, all } => {
            let bundle = if all {
                install::render_unlink_all_bundle().expect("unlink-all bundle should render")
            } else {
                let provider = provider
                    .as_deref()
                    .expect("provider should be passed unless --all is used");
                let provider = ProviderKind::from_slug(provider).expect("provider should be valid");
                install::render_unlink_bundle(provider).expect("unlink bundle should render")
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&bundle).expect("unlink bundle should serialize")
            );
        }
        Commands::Uninstall => {
            let bundle = install::render_uninstall_bundle().expect("uninstall bundle should render");
            println!(
                "{}",
                serde_json::to_string_pretty(&bundle).expect("uninstall bundle should serialize")
            );
        }
        Commands::Serve => mcp::serve_stdio().expect("serve should succeed"),
        Commands::Resume { hash_id } => {
            refresh_runtime_state("resume");
            let storage = Storage::open(configured_db_path().expect("db path should resolve"))
                .expect("storage should open");
            match storage
                .resume_bundle(&hash_id)
                .expect("resume query should succeed")
            {
                Some(bundle) => println!("{bundle}"),
                None => {
                    println!("no conversation found for {hash_id}");
                    let matches = find_exact_source_matches(&storage, &hash_id)
                        .expect("source match lookup should succeed");
                    if !matches.is_empty() {
                        println!("exact source matches:");
                        for source_match in matches {
                            println!("{source_match}");
                        }
                    }
                }
            }
        }
        Commands::Forget { provider, hash_id } => {
            refresh_runtime_state("forget");
            let provider = ProviderKind::from_slug(&provider).expect("provider should be valid");
            let storage = Storage::open(configured_db_path().expect("db path should resolve"))
                .expect("storage should open");
            if storage
                .forget_conversation(provider, &hash_id)
                .expect("forget should succeed")
            {
                println!("forgot {} {}", provider.as_slug(), hash_id);
            } else {
                println!("no conversation found for {} {}", provider.as_slug(), hash_id);
            }
        }
        Commands::ConvSearch { query } => {
            refresh_runtime_state("conv-search");
            let storage = Storage::open(configured_db_path().expect("db path should resolve"))
                .expect("storage should open");
            eprintln!("cmb: searching indexed conversation history...");
            let results = storage
                .search_conversations(&query, 10)
                .expect("conversation search should succeed");
            if results.is_empty() {
                println!("no conversations matched {query}");
            } else {
                println!("{}", results.join("\n"));
            }
        }
        Commands::Doctor => {
            refresh_runtime_state("doctor");
            let status = doctor::inspect().expect("doctor should succeed");
            println!(
                "{}",
                serde_json::to_string_pretty(&status).expect("doctor output should serialize")
            );
        }
        Commands::Stats => {
            refresh_runtime_state("stats");
            let value = mcp::memory_stats(configured_db_path().expect("db path should resolve"))
                .expect("stats should succeed");
            println!(
                "{}",
                serde_json::to_string_pretty(&value).expect("stats output should serialize")
            );
        }
    }
}

fn refresh_runtime_state(command_name: &str) {
    eprintln!("cmb: refreshing local conversation sources before {command_name}...");
    run_refresh().expect("refresh before command should succeed");
}

fn find_exact_source_matches(storage: &Storage, needle: &str) -> anyhow::Result<Vec<String>> {
    let mut matches = BTreeSet::new();
    for source in storage.known_source_locations()? {
        let path = PathBuf::from(&source.source_path);
        if path.is_file() {
            if file_contains_exact(&path, needle) {
                matches.insert(format!("[{}] {}", source.provider.as_slug(), path.display()));
            }
            continue;
        }

        if path.is_dir() {
            collect_exact_matches(&path, needle, source.provider.as_slug(), &mut matches)?;
        }
    }

    Ok(matches.into_iter().collect())
}

fn collect_exact_matches(
    dir: &Path,
    needle: &str,
    provider_slug: &str,
    out: &mut BTreeSet<String>,
) -> anyhow::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_exact_matches(&path, needle, provider_slug, out)?;
        } else if file_contains_exact(&path, needle) {
            out.insert(format!("[{provider_slug}] {}", path.display()));
        }
    }

    Ok(())
}

fn file_contains_exact(path: &Path, needle: &str) -> bool {
    match fs::read(path) {
        Ok(bytes) => String::from_utf8_lossy(&bytes).contains(needle),
        Err(_) => false,
    }
}
