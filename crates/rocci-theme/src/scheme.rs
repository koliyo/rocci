use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ColorSchemePolicy {
    Light,
    Dark,
    #[default]
    Auto,
}

impl ColorSchemePolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
            Self::Auto => "auto",
        }
    }

    pub fn meta_content(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
            Self::Auto => "light dark",
        }
    }

    pub fn html_attr(self) -> Option<&'static str> {
        match self {
            Self::Light => Some("light"),
            Self::Dark => Some("dark"),
            Self::Auto => None,
        }
    }
}

impl fmt::Display for ColorSchemePolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ColorSchemePolicy {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "light" => Ok(Self::Light),
            "dark" => Ok(Self::Dark),
            "auto" => Ok(Self::Auto),
            other => Err(format!(
                "unknown color scheme `{other}`; expected light, dark, or auto"
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_color_scheme_policy() {
        assert_eq!(
            "light".parse::<ColorSchemePolicy>().unwrap(),
            ColorSchemePolicy::Light
        );
        assert_eq!(
            "dark".parse::<ColorSchemePolicy>().unwrap(),
            ColorSchemePolicy::Dark
        );
        assert_eq!(
            "auto".parse::<ColorSchemePolicy>().unwrap(),
            ColorSchemePolicy::Auto
        );
        assert!("sepia".parse::<ColorSchemePolicy>().is_err());
    }
}
