use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LanguageId {
    Roc,
    Html,
    Css,
    Rocci,
    Rocdown,
    Markdown,
    Shell,
    Toml,
    PlainText,
    Other(String),
}

impl LanguageId {
    pub fn parse(s: &str) -> Self {
        let trimmed = s.trim();
        match trimmed.to_ascii_lowercase().as_str() {
            "roc" => Self::Roc,
            "html" | "htm" => Self::Html,
            "css" => Self::Css,
            "rocci" => Self::Rocci,
            "rocdown" => Self::Rocdown,
            "md" | "markdown" => Self::Markdown,
            "sh" | "bash" | "shell" | "zsh" => Self::Shell,
            "toml" => Self::Toml,
            "text" | "txt" | "plain" => Self::PlainText,
            "" => Self::PlainText,
            _ => Self::Other(trimmed.to_string()),
        }
    }

    pub fn canonical_name(&self) -> &str {
        match self {
            Self::Roc => "roc",
            Self::Html => "html",
            Self::Css => "css",
            Self::Rocci => "rocci",
            Self::Rocdown => "rocdown",
            Self::Markdown => "markdown",
            Self::Shell => "shell",
            Self::Toml => "toml",
            Self::PlainText => "text",
            Self::Other(s) => s.as_str(),
        }
    }

    pub fn is_highlighted(&self) -> bool {
        matches!(
            self,
            Self::Roc | Self::Html | Self::Css | Self::Rocci | Self::Rocdown | Self::Markdown
        )
    }
}

impl std::fmt::Display for LanguageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.canonical_name())
    }
}

impl From<&str> for LanguageId {
    fn from(s: &str) -> Self {
        Self::parse(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_aliases() {
        assert_eq!(LanguageId::parse("ROC"), LanguageId::Roc);
        assert_eq!(LanguageId::parse("sh"), LanguageId::Shell);
        assert_eq!(LanguageId::parse("bash"), LanguageId::Shell);
        assert_eq!(LanguageId::parse("htm"), LanguageId::Html);
        assert_eq!(LanguageId::parse("html"), LanguageId::Html);
        assert_eq!(LanguageId::parse("md"), LanguageId::Markdown);
        assert_eq!(LanguageId::parse("rocdown"), LanguageId::Rocdown);
        assert_eq!(LanguageId::parse("rocci"), LanguageId::Rocci);
        assert_eq!(
            LanguageId::parse("unknown_lang"),
            LanguageId::Other("unknown_lang".to_string())
        );
    }
}
