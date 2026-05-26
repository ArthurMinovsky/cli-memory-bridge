//! Integrations library crate.

mod assets;
mod detect;
mod importers;
mod installers;

pub use assets::{
    render_claude_conv_search_command, render_claude_forget_command,
    render_claude_resume_command, render_codex_conv_search_skill, render_codex_forget_skill,
    render_codex_resume_skill,
};
pub use detect::{DetectedProvider, detect_providers};
pub use importers::{ImportedTranscript, import_claude, import_codex};
pub use importers::{
    import_antigravity_cli, import_copilot, import_gemini, import_hermes, import_opencode,
    import_zed,
};
pub use installers::{
    render_codex_install, render_copilot_install, render_gemini_install, render_hermes_install,
    render_opencode_install, render_zed_install,
};
