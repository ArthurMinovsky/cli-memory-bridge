use std::{error::Error, fmt, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderKind {
    Codex,
    Claude,
    Hermes,
    Gemini,
    OpenCode,
    Copilot,
    Zed,
    AntigravityCli,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderParseError {
    slug: String,
}

impl ProviderParseError {
    pub fn slug(&self) -> &str {
        &self.slug
    }
}

impl fmt::Display for ProviderParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown provider slug: {}", self.slug)
    }
}

impl Error for ProviderParseError {}

impl ProviderKind {
    pub const fn as_slug(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::Claude => "claude",
            Self::Hermes => "hermes",
            Self::Gemini => "gemini",
            Self::OpenCode => "opencode",
            Self::Copilot => "copilot",
            Self::Zed => "zed",
            Self::AntigravityCli => "antigravity-cli",
        }
    }

    pub fn from_slug(slug: &str) -> Result<Self, ProviderParseError> {
        match slug {
            "codex" => Ok(Self::Codex),
            "claude" => Ok(Self::Claude),
            "hermes" => Ok(Self::Hermes),
            "gemini" => Ok(Self::Gemini),
            "opencode" => Ok(Self::OpenCode),
            "copilot" => Ok(Self::Copilot),
            "zed" => Ok(Self::Zed),
            "antigravity-cli" => Ok(Self::AntigravityCli),
            _ => Err(ProviderParseError {
                slug: slug.to_owned(),
            }),
        }
    }
}

impl TryFrom<&str> for ProviderKind {
    type Error = ProviderParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_slug(value)
    }
}

impl FromStr for ProviderKind {
    type Err = ProviderParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_slug(s)
    }
}
