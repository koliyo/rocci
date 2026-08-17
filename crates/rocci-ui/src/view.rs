use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SiteView {
    pub title: String,
    pub description: String,
    pub base_url: String,
    pub language: String,
    pub repository: String,
    pub social_image: String,
    pub subtitle: String,
    pub footer: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct LaneView {
    pub label: String,
    pub href: String,
    pub current: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct NavItemView {
    pub title: String,
    pub href: String,
    pub class_name: String,
}

impl NavItemView {
    pub fn new(
        title: impl Into<String>,
        href: impl Into<String>,
        class_name: impl Into<String>,
    ) -> Self {
        Self {
            title: title.into(),
            href: href.into(),
            class_name: class_name.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct BreadcrumbView {
    pub title: String,
    pub href: String,
}

impl BreadcrumbView {
    pub fn new(title: impl Into<String>, href: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            href: href.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct OutlineView {
    pub id: String,
    pub title: String,
    pub level: String,
}

impl OutlineView {
    pub fn new(id: impl Into<String>, title: impl Into<String>, level: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            level: level.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ResourceView {
    pub stylesheet: String,
    pub csp: String,
    pub canonical: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PageView {
    pub site: SiteView,
    pub lanes: Vec<LaneView>,
    pub sidebar: Vec<NavItemView>,
    pub route: String,
    pub title: String,
    pub description: String,
    pub outline: Vec<OutlineView>,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub previous: NavItemView,
    pub next: NavItemView,
    pub resources: ResourceView,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StatTone {
    #[default]
    Default,
    Action,
    Positive,
    Warning,
    Danger,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct StatCardView {
    pub value: String,
    pub label: String,
    pub tone: StatTone,
    pub href: Option<String>,
}

impl StatCardView {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            tone: StatTone::Default,
            href: None,
        }
    }

    pub fn with_tone(mut self, tone: StatTone) -> Self {
        self.tone = tone;
        self
    }

    pub fn with_href(mut self, href: impl Into<String>) -> Self {
        self.href = Some(href.into());
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BadgeTone {
    #[default]
    Default,
    Type,
    Draft,
    Stable,
    Deprecated,
    Human,
    Generated,
    Unverified,
    AuthNormative,
    AuthExploratory,
    AuthDescriptive,
    ActionClean,
    ActionRequired,
    ActionError,
    ActionInfo,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct BadgeView {
    pub label: String,
    pub tone: BadgeTone,
    pub sub_label: Option<String>,
}

impl BadgeView {
    pub fn new(label: impl Into<String>, tone: BadgeTone) -> Self {
        Self {
            label: label.into(),
            tone,
            sub_label: None,
        }
    }

    pub fn with_sub_label(mut self, sub: impl Into<String>) -> Self {
        self.sub_label = Some(sub.into());
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AlertTone {
    #[default]
    Warning,
    Info,
    Danger,
    Success,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct AlertView {
    pub icon: String,
    pub title: String,
    pub message: String,
    pub tone: AlertTone,
}

impl AlertView {
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            icon: "⚠️".to_string(),
            title: title.into(),
            message: message.into(),
            tone: AlertTone::Warning,
        }
    }

    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = icon.into();
        self
    }

    pub fn with_tone(mut self, tone: AlertTone) -> Self {
        self.tone = tone;
        self
    }
}
