use askama::Template;
use okf::Bundle;

#[derive(Clone, Debug, Default)]
pub struct NavNode {
    pub href: String,
    pub title: String,
    pub current: bool,
    pub open: bool,
    pub children: Vec<NavNode>,
}

#[derive(Clone, Debug)]
pub struct TocEntry {
    pub id: String,
    pub text: String,
    pub level: u8,
}

#[derive(Clone, Debug)]
pub struct SettingsRoot {
    pub id: String,
    pub kind: String,
    pub detail: String,
    pub incoming: String,
    pub token_env: String,
    pub has_token: bool,
    pub warning: String,
}

#[derive(Clone, Debug)]
pub struct ReviewRow {
    pub href: String,
    pub title: String,
    pub id: String,
    pub status: String,
    pub action: String,
}

macro_rules! document_template {
    ($name:ident, $path:literal) => {
        #[derive(Template)]
        #[template(path = $path)]
        pub struct $name {
            pub title: String,
            pub nav: Vec<NavNode>,
            pub toc: Vec<TocEntry>,
            pub article_html: String,
            pub concept_type: String,
            pub status: String,
            pub authority: String,
            pub review_rows: Vec<ReviewRow>,
            pub message: String,
            pub config_path: String,
            pub settings_roots: Vec<SettingsRoot>,
        }
    };
}

document_template!(PageTemplate, "page.html");
document_template!(HomeTemplate, "home.html");
document_template!(ReviewTemplate, "review.html");
document_template!(SettingsTemplate, "settings.html");
document_template!(SettingsFragmentTemplate, "fragments/settings.html");

pub struct Document {
    pub title: String,
    pub nav: Vec<NavNode>,
    pub toc: Vec<TocEntry>,
    pub article_html: String,
    pub concept_type: String,
    pub status: String,
    pub authority: String,
    pub review_rows: Vec<ReviewRow>,
    pub message: String,
    pub config_path: String,
    pub settings_roots: Vec<SettingsRoot>,
}

impl Document {
    pub fn render_page(self) -> askama::Result<String> {
        PageTemplate::from(self).render()
    }

    pub fn render_home(self) -> askama::Result<String> {
        HomeTemplate::from(self).render()
    }

    pub fn render_review(self) -> askama::Result<String> {
        ReviewTemplate::from(self).render()
    }

    pub fn render_settings(self) -> askama::Result<String> {
        SettingsTemplate::from(self).render()
    }

    pub fn render_settings_fragment(self) -> askama::Result<String> {
        SettingsFragmentTemplate::from(self).render()
    }
}

impl From<Document> for PageTemplate {
    fn from(document: Document) -> Self {
        Self {
            title: document.title,
            nav: document.nav,
            toc: document.toc,
            article_html: document.article_html,
            concept_type: document.concept_type,
            status: document.status,
            authority: document.authority,
            review_rows: document.review_rows,
            message: document.message,
            config_path: document.config_path,
            settings_roots: document.settings_roots,
        }
    }
}

impl From<Document> for HomeTemplate {
    fn from(document: Document) -> Self {
        Self {
            title: document.title,
            nav: document.nav,
            toc: document.toc,
            article_html: document.article_html,
            concept_type: document.concept_type,
            status: document.status,
            authority: document.authority,
            review_rows: document.review_rows,
            message: document.message,
            config_path: document.config_path,
            settings_roots: document.settings_roots,
        }
    }
}

impl From<Document> for ReviewTemplate {
    fn from(document: Document) -> Self {
        Self {
            title: document.title,
            nav: document.nav,
            toc: document.toc,
            article_html: document.article_html,
            concept_type: document.concept_type,
            status: document.status,
            authority: document.authority,
            review_rows: document.review_rows,
            message: document.message,
            config_path: document.config_path,
            settings_roots: document.settings_roots,
        }
    }
}

impl From<Document> for SettingsTemplate {
    fn from(document: Document) -> Self {
        Self {
            title: document.title,
            nav: document.nav,
            toc: document.toc,
            article_html: document.article_html,
            concept_type: document.concept_type,
            status: document.status,
            authority: document.authority,
            review_rows: document.review_rows,
            message: document.message,
            config_path: document.config_path,
            settings_roots: document.settings_roots,
        }
    }
}

impl From<Document> for SettingsFragmentTemplate {
    fn from(document: Document) -> Self {
        Self {
            title: document.title,
            nav: document.nav,
            toc: document.toc,
            article_html: document.article_html,
            concept_type: document.concept_type,
            status: document.status,
            authority: document.authority,
            review_rows: document.review_rows,
            message: document.message,
            config_path: document.config_path,
            settings_roots: document.settings_roots,
        }
    }
}

pub fn toc_from_headings(headings: &[okf::Heading]) -> Vec<TocEntry> {
    headings
        .iter()
        .filter(|heading| (2..=3).contains(&heading.level))
        .map(|heading| TocEntry {
            id: heading.id.clone(),
            text: heading.text.clone(),
            level: heading.level,
        })
        .collect()
}

pub fn review_rows(bundle: &Bundle) -> Vec<ReviewRow> {
    bundle
        .concepts
        .iter()
        .map(|concept| {
            let action = okf::classify_concept_action(concept, &bundle.diagnostics);
            ReviewRow {
                href: format!("/{}/", concept.id),
                title: okf::string_field(&concept.metadata, "title")
                    .unwrap_or(&concept.id)
                    .to_string(),
                id: concept.id.clone(),
                status: okf::string_field(&concept.metadata, "status")
                    .unwrap_or("draft")
                    .to_string(),
                action: action.label,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_document(toc: Vec<TocEntry>) -> Document {
        Document {
            title: "Hello".into(),
            nav: vec![
                NavNode {
                    href: "/".into(),
                    title: "Dashboard".into(),
                    current: true,
                    open: false,
                    children: Vec::new(),
                },
                NavNode {
                    href: "/review/".into(),
                    title: "Review queue".into(),
                    current: false,
                    open: false,
                    children: Vec::new(),
                },
            ],
            toc,
            article_html: "<h1>Hello</h1><p>Body</p>".into(),
            concept_type: "Architecture".into(),
            status: "draft".into(),
            authority: "descriptive".into(),
            review_rows: Vec::new(),
            message: String::new(),
            config_path: "~/.okmate/config.toml".into(),
            settings_roots: Vec::new(),
        }
    }

    #[test]
    fn page_template_contains_shell_landmarks() {
        let html = sample_document(vec![TocEntry {
            id: "section".into(),
            text: "Section".into(),
            level: 2,
        }])
        .render_page()
        .unwrap();
        assert!(html.contains("id=\"okmate-nav\""), "{html}");
        assert!(html.contains("id=\"okmate-main\""), "{html}");
        assert!(html.contains("id=\"okmate-toc\""), "{html}");
        assert!(html.contains("<h1>Hello</h1>"));
        assert!(html.contains("/__okmate/app.css"));
    }

    #[test]
    fn settings_template_is_empty_state() {
        let html = sample_document(Vec::new()).render_settings().unwrap();
        assert!(html.contains("id=\"okmate-settings\""));
        assert!(html.contains("id=\"okmate-nav\""));
        assert!(html.contains("No roots yet"));
    }

    #[test]
    fn review_template_contains_queue_region() {
        let mut document = sample_document(Vec::new());
        document.review_rows = vec![ReviewRow {
            href: "/hello/".into(),
            title: "Hello".into(),
            id: "hello".into(),
            status: "draft".into(),
            action: "Clean".into(),
        }];
        let html = document.render_review().unwrap();
        assert!(html.contains("id=\"okmate-queue\""));
        assert!(html.contains("Hello"));
    }
}
