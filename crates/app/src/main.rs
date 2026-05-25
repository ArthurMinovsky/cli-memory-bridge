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
            println!(
                "detected {} providers, checkpointed {} sources, imported {} conversations / {} messages",
                summary.provider_count, summary.checkpoint_count
                , summary.imported_conversations, summary.imported_messages
            );
            if !summary.providers.is_empty() {
                println!("{}", summary.providers.join(","));
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
            eprintln!("cli-memory: refreshing local conversation sources...");
            run_refresh().expect("refresh before resume should succeed");
            let storage = Storage::open(configured_db_path().expect("db path should resolve"))
                .expect("storage should open");
            match storage
                .resume_bundle(&hash_id)
                .expect("resume query should succeed")
            {
                Some(bundle) => println!("{bundle}"),
                None => println!("no conversation found for {hash_id}"),
            }
        }
        Commands::Forget { provider, hash_id } => {
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
            eprintln!("cli-memory: refreshing local conversation sources...");
            run_refresh().expect("refresh before conversation search should succeed");
            let storage = Storage::open(configured_db_path().expect("db path should resolve"))
                .expect("storage should open");
            eprintln!("cli-memory: searching indexed conversation history...");
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
            let status = doctor::inspect().expect("doctor should succeed");
            println!(
                "{}",
                serde_json::to_string_pretty(&status).expect("doctor output should serialize")
            );
        }
        Commands::Stats => {
            let value = mcp::memory_stats(configured_db_path().expect("db path should resolve"))
                .expect("stats should succeed");
            println!(
                "{}",
                serde_json::to_string_pretty(&value).expect("stats output should serialize")
            );
        }
    }
}
