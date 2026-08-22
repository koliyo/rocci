use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SiteView {
    pub title: String,
    pub description: String,
    pub base_url: String,
    pub language: String,
    pub repository: String,
    pub social_image: String,
    pub favicon: String,
    pub apple_touch_icon: String,
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
pub struct NavGroupView {
    pub title: String,
    pub href: String,
    pub open: bool,
    pub items: Vec<NavItemView>,
}

impl NavGroupView {
    pub fn new(
        title: impl Into<String>,
        href: impl Into<String>,
        open: bool,
        items: Vec<NavItemView>,
    ) -> Self {
        Self {
            title: title.into(),
            href: href.into(),
            open,
            items,
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
    #[serde(default)]
    pub module_script: String,
    #[serde(default)]
    pub chrome_script: String,
    #[serde(default)]
    pub playground_css: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CollectionItemView {
    pub route: String,
    pub title: String,
    pub summary: String,
    pub published: String,
    pub updated: String,
    pub authors: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PageView {
    pub site: SiteView,
    pub lanes: Vec<LaneView>,
    pub sidebar: Vec<NavGroupView>,
    pub route: String,
    pub title: String,
    pub document_title: String,
    pub description: String,
    pub layout: String,
    pub published: String,
    pub updated: String,
    pub authors: Vec<String>,
    pub tags: Vec<String>,
    pub collection: String,
    pub collection_items: Vec<CollectionItemView>,
    pub outline: Vec<OutlineView>,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub previous: NavItemView,
    pub next: NavItemView,
    pub resources: ResourceView,
}
