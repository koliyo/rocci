use crate::view::{
    AlertTone, AlertView, BadgeTone, BadgeView, BreadcrumbView, LaneView, NavItemView, OutlineView,
    PageView, StatCardView, StatTone,
};

pub fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

pub fn render_site_shell(view: &PageView, content: &str) -> String {
    let mut out = String::new();
    let lang = if view.site.language.is_empty() {
        "en"
    } else {
        &view.site.language
    };

    out.push_str("<!doctype html>\n");
    out.push_str(&format!("<html lang=\"{}\">\n", escape(lang)));
    out.push_str("<head>\n");
    out.push_str("  <meta charset=\"utf-8\">\n");
    out.push_str("  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
    out.push_str("  <meta name=\"color-scheme\" content=\"light dark\">\n");
    if !view.resources.csp.is_empty() {
        out.push_str(&format!(
            "  <meta http-equiv=\"Content-Security-Policy\" content=\"{}\">\n",
            escape(&view.resources.csp)
        ));
    }
    let full_title = if view.site.title.is_empty() {
        view.title.clone()
    } else if view.title.is_empty() {
        view.site.title.clone()
    } else {
        format!("{} · {}", view.title, view.site.title)
    };
    out.push_str(&format!("  <title>{}</title>\n", escape(&full_title)));
    if !view.resources.stylesheet.is_empty() {
        out.push_str(&format!(
            "  <link rel=\"stylesheet\" href=\"{}\">\n",
            escape(&view.resources.stylesheet)
        ));
    }
    let desc = if !view.description.is_empty() {
        &view.description
    } else {
        &view.site.description
    };
    if !desc.is_empty() {
        out.push_str(&format!(
            "  <meta name=\"description\" content=\"{}\">\n",
            escape(desc)
        ));
    }
    if !view.resources.canonical.is_empty() {
        out.push_str(&format!(
            "  <link rel=\"canonical\" href=\"{}\">\n",
            escape(&view.resources.canonical)
        ));
        out.push_str(&format!(
            "  <meta property=\"og:url\" content=\"{}\">\n",
            escape(&view.resources.canonical)
        ));
    }
    out.push_str("  <meta property=\"og:type\" content=\"website\">\n");
    out.push_str(&format!(
        "  <meta property=\"og:title\" content=\"{}\">\n",
        escape(&view.title)
    ));
    if !view.site.title.is_empty() {
        out.push_str(&format!(
            "  <meta property=\"og:site_name\" content=\"{}\">\n",
            escape(&view.site.title)
        ));
    }
    if !desc.is_empty() {
        out.push_str(&format!(
            "  <meta property=\"og:description\" content=\"{}\">\n",
            escape(desc)
        ));
    }
    if !view.site.social_image.is_empty() {
        let img_url = format!("{}{}", view.site.base_url, view.site.social_image);
        out.push_str(&format!(
            "  <meta property=\"og:image\" content=\"{}\">\n",
            escape(&img_url)
        ));
        out.push_str("  <meta name=\"twitter:card\" content=\"summary_large_image\">\n");
    }
    out.push_str("</head>\n");
    out.push_str("<body>\n");
    out.push_str("  <a class=\"skip-link\" href=\"#main-content\">Skip to content</a>\n");

    // Header
    out.push_str("  <header class=\"site-header\">\n");
    out.push_str("    <div class=\"header-inner\">\n");
    out.push_str(&format!(
        "      <a class=\"brand\" href=\"/\" aria-label=\"{} home\">\n",
        escape(&view.site.title)
    ));
    out.push_str("        <span class=\"brand-mark\" aria-hidden=\"true\">r</span>\n");
    out.push_str(&format!(
        "        <span>{}</span>\n",
        escape(&view.site.title)
    ));
    out.push_str("      </a>\n");
    if !view.site.subtitle.is_empty() {
        out.push_str("      <span class=\"header-rule\" aria-hidden=\"true\"></span>\n");
        out.push_str(&format!(
            "      <span class=\"header-subtitle\">{}</span>\n",
            escape(&view.site.subtitle)
        ));
    }
    if !view.lanes.is_empty() {
        out.push_str(&render_nav_lanes(&view.lanes));
    }
    out.push_str("      <div class=\"header-actions\">\n");
    if !view.site.repository.is_empty() {
        out.push_str(&format!(
            "        <a class=\"header-link\" href=\"{}\">Source ↗</a>\n",
            escape(&view.site.repository)
        ));
    }
    out.push_str("      </div>\n");

    // Mobile menu drawer
    out.push_str("      <details class=\"mobile-menu\">\n");
    out.push_str("        <summary>Menu</summary>\n");
    out.push_str("        <div class=\"mobile-panel\">\n");
    if !view.lanes.is_empty() {
        out.push_str(&render_nav_lanes(&view.lanes));
    }
    out.push_str("          <nav aria-label=\"Site navigation\">\n");
    out.push_str("            <div class=\"nav-items\">\n");
    for item in &view.sidebar {
        let is_cur = item.href == view.route;
        let cur_attr = if is_cur { " aria-current=\"page\"" } else { "" };
        out.push_str(&format!(
            "              <a class=\"{}\" href=\"{}\"{}>{}</a>\n",
            escape(&item.class_name),
            escape(&item.href),
            cur_attr,
            escape(&item.title)
        ));
    }
    out.push_str("            </div>\n");
    out.push_str("          </nav>\n");
    out.push_str("        </div>\n");
    out.push_str("      </details>\n");
    out.push_str("    </div>\n");
    out.push_str("  </header>\n");

    // Site Grid
    out.push_str("  <div class=\"site-grid\">\n");
    out.push_str("    <aside class=\"sidebar\">\n");
    out.push_str("      <nav aria-label=\"Site navigation\">\n");
    out.push_str("        <div class=\"nav-items\">\n");
    for item in &view.sidebar {
        let is_cur = item.href == view.route;
        let cur_attr = if is_cur { " aria-current=\"page\"" } else { "" };
        out.push_str(&format!(
            "          <a class=\"{}\" href=\"{}\"{}>{}</a>\n",
            escape(&item.class_name),
            escape(&item.href),
            cur_attr,
            escape(&item.title)
        ));
    }
    out.push_str("        </div>\n");
    out.push_str("      </nav>\n");
    out.push_str("    </aside>\n");

    // Content Column
    out.push_str("    <main class=\"content-column\" id=\"main-content\">\n");
    if !view.breadcrumbs.is_empty() {
        out.push_str(&render_breadcrumbs(&view.breadcrumbs, &view.route));
    }
    out.push_str("      <article class=\"article\">\n");
    out.push_str(content);
    out.push_str("\n      </article>\n");

    if !view.previous.href.is_empty() || !view.next.href.is_empty() {
        out.push_str(&render_journey(&view.previous, &view.next));
    }
    if !view.site.footer.is_empty() {
        out.push_str(&format!(
            "      <footer class=\"article-footer\">{}</footer>\n",
            escape(&view.site.footer)
        ));
    }
    out.push_str("    </main>\n");

    // Outline / TOC
    out.push_str("    <aside class=\"outline\" aria-label=\"On this page\">\n");
    if !view.outline.is_empty() {
        out.push_str(&render_outline(&view.outline));
    }
    out.push_str("    </aside>\n");
    out.push_str("  </div>\n");
    out.push_str("</body>\n");
    out.push_str("</html>\n");

    out
}

pub fn render_nav_lanes(lanes: &[LaneView]) -> String {
    let mut out = String::new();
    out.push_str("      <nav class=\"lanes\" aria-label=\"Sections\">\n");
    for lane in lanes {
        let (class_name, cur_attr) = if lane.current {
            ("lane-link is-current", " aria-current=\"true\"")
        } else {
            ("lane-link", " aria-current=\"false\"")
        };
        out.push_str(&format!(
            "        <a class=\"{}\" href=\"{}\"{}>{}</a>\n",
            class_name,
            escape(&lane.href),
            cur_attr,
            escape(&lane.label)
        ));
    }
    out.push_str("      </nav>\n");
    out
}

pub fn render_breadcrumbs(breadcrumbs: &[BreadcrumbView], current_route: &str) -> String {
    let mut out = String::new();
    out.push_str("      <nav class=\"breadcrumbs\" aria-label=\"Breadcrumb\">\n");
    out.push_str("        <ol>\n");
    for crumb in breadcrumbs {
        let is_cur = crumb.href == current_route;
        let class_attr = if is_cur {
            " class=\"crumb-current\""
        } else {
            ""
        };
        let cur_attr = if is_cur {
            " aria-current=\"page\""
        } else {
            " aria-current=\"false\""
        };
        out.push_str(&format!(
            "          <li><a{} href=\"{}\"{}>{}</a></li>\n",
            class_attr,
            escape(&crumb.href),
            cur_attr,
            escape(&crumb.title)
        ));
    }
    out.push_str("        </ol>\n");
    out.push_str("      </nav>\n");
    out
}

pub fn render_outline(outline: &[OutlineView]) -> String {
    let mut out = String::new();
    out.push_str("      <p class=\"outline-label\">On this page</p>\n");
    out.push_str("      <div class=\"outline-items\">\n");
    for heading in outline {
        let class_name = if heading.level == "3" {
            "outline-link level-3"
        } else {
            "outline-link"
        };
        out.push_str(&format!(
            "        <a class=\"{}\" href=\"#{}\">{}</a>\n",
            class_name,
            escape(&heading.id),
            escape(&heading.title)
        ));
    }
    out.push_str("      </div>\n");
    out
}

pub fn render_journey(previous: &NavItemView, next: &NavItemView) -> String {
    let mut out = String::new();
    out.push_str("      <nav class=\"journey\" aria-label=\"Page\">\n");
    if !previous.href.is_empty() {
        out.push_str(&format!(
            "        <a class=\"journey-prev\" href=\"{}\">&larr; {}</a>\n",
            escape(&previous.href),
            escape(&previous.title)
        ));
    }
    if !next.href.is_empty() {
        out.push_str(&format!(
            "        <a class=\"journey-next\" href=\"{}\">{} &rarr;</a>\n",
            escape(&next.href),
            escape(&next.title)
        ));
    }
    out.push_str("      </nav>\n");
    out
}

pub fn render_stat_grid(cards: &[StatCardView]) -> String {
    let mut out = String::new();
    out.push_str("<div class=\"okf-stat-grid\">\n");
    for card in cards {
        let tone_class = match card.tone {
            StatTone::Default => "",
            StatTone::Action => " is-action",
            StatTone::Positive => " is-positive",
            StatTone::Warning => " is-warning",
            StatTone::Danger => " is-danger",
        };
        if let Some(href) = &card.href {
            out.push_str(&format!(
                "  <a href=\"{}\" class=\"okf-stat-card{}\"><div class=\"okf-stat-value\">{}</div><div class=\"okf-stat-label\">{}</div></a>\n",
                escape(href),
                tone_class,
                escape(&card.value),
                escape(&card.label)
            ));
        } else {
            out.push_str(&format!(
                "  <div class=\"okf-stat-card{}\"><div class=\"okf-stat-value\">{}</div><div class=\"okf-stat-label\">{}</div></div>\n",
                tone_class,
                escape(&card.value),
                escape(&card.label)
            ));
        }
    }
    out.push_str("</div>\n");
    out
}

pub fn render_badge(badge: &BadgeView) -> String {
    let tone_class = match badge.tone {
        BadgeTone::Default => "okf-badge",
        BadgeTone::Type => "okf-badge okf-type",
        BadgeTone::Draft => "okf-badge okf-status okf-status-draft",
        BadgeTone::Stable => "okf-badge okf-status okf-status-stable",
        BadgeTone::Deprecated => "okf-badge okf-status okf-status-deprecated",
        BadgeTone::Human => "okf-badge okf-trust okf-trust-human",
        BadgeTone::Generated => "okf-badge okf-trust okf-trust-generated",
        BadgeTone::Unverified => "okf-badge okf-trust okf-trust-unverified",
        BadgeTone::AuthNormative => "okf-badge okf-auth okf-auth-normative",
        BadgeTone::AuthExploratory => "okf-badge okf-auth okf-auth-exploratory",
        BadgeTone::AuthDescriptive => "okf-badge okf-auth okf-auth-descriptive",
        BadgeTone::ActionClean => "okf-action-pill pill-clean",
        BadgeTone::ActionRequired => "okf-action-pill pill-action",
        BadgeTone::ActionError => "okf-action-pill pill-error",
        BadgeTone::ActionInfo => "okf-action-pill pill-info",
    };
    if let Some(sub) = &badge.sub_label {
        format!(
            "<span class=\"{}\">{}</span><div class=\"okf-badge-sub\"><code>{}</code></div>",
            tone_class,
            escape(&badge.label),
            escape(sub)
        )
    } else {
        format!(
            "<span class=\"{}\">{}</span>",
            tone_class,
            escape(&badge.label)
        )
    }
}

pub fn render_alert_banner(alert: &AlertView) -> String {
    let tone_class = match alert.tone {
        AlertTone::Warning => "okf-alert-banner alert-warning",
        AlertTone::Info => "okf-alert-banner alert-info",
        AlertTone::Danger => "okf-alert-banner alert-danger",
        AlertTone::Success => "okf-alert-banner alert-success",
    };
    format!(
        "<div class=\"{}\" role=\"alert\">\n  <span class=\"okf-alert-icon\" aria-hidden=\"true\">{}</span>\n  <div class=\"okf-alert-content\">\n    <strong>{}:</strong> {}\n  </div>\n</div>\n",
        tone_class,
        escape(&alert.icon),
        escape(&alert.title),
        escape(&alert.message)
    )
}
