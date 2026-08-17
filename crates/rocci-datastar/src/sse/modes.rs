use std::fmt;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PatchMode {
    #[default]
    Outer,
    Inner,
    Before,
    After,
    Prepend,
    Append,
    UpsertAttributes,
}

impl PatchMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Outer => "outer",
            Self::Inner => "inner",
            Self::Before => "before",
            Self::After => "after",
            Self::Prepend => "prepend",
            Self::Append => "append",
            Self::UpsertAttributes => "upsertAttributes",
        }
    }
}

impl fmt::Display for PatchMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
