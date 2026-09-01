use crate::catalog::{self, NavLink, NavSection, PageHeading, ResolvedPage};
use rocci_ui::{BreadcrumbView, LaneView, NavGroupView, NavItemView, OutlineView, SiteView};

pub(crate) fn lanes_and_sidebar(
    navigation: &[NavSection],
    current_id: Option<&str>,
) -> (Vec<LaneView>, Vec<NavGroupView>) {
    let has_nested = navigation
        .iter()
        .any(|section| !section.children.is_empty());
    let current_top = current_id.and_then(|id| current_section(navigation, id));
    let lanes = if has_nested {
        navigation
            .iter()
            .map(|section| LaneView {
                label: section.label.clone(),
                href: catalog::first_nav_item(section)
                    .map(|item| item.route.clone())
                    .unwrap_or_else(|| "/".into()),
                current: current_top.is_some_and(|current| current.label == section.label),
            })
            .collect()
    } else {
        Vec::new()
    };
    let groups: Vec<&NavSection> = if has_nested {
        current_top
            .map(|section| {
                if section.children.is_empty() {
                    vec![section]
                } else {
                    section.children.iter().collect()
                }
            })
            .unwrap_or_default()
    } else {
        navigation.iter().collect()
    };
    let sidebar = groups
        .into_iter()
        .filter_map(|section| nav_group_view(section, current_id))
        .collect();
    (lanes, sidebar)
}

fn current_section<'a>(navigation: &'a [NavSection], id: &str) -> Option<&'a NavSection> {
    navigation.iter().find(|section| {
        catalog::section_contains(section, id)
            || catalog::first_nav_item(section).is_some_and(|item| {
                item.id
                    .split('/')
                    .next()
                    .zip(id.split('/').next())
                    .is_some_and(|(lane, current)| lane == current)
            })
    })
}

pub(crate) fn sidebar_has_current(sidebar: &[NavGroupView], route: &str) -> bool {
    sidebar.iter().any(|group| {
        group.covers_href(route)
            || group.marks_current()
            || (group.items.is_empty() && group.children.is_empty() && group.open)
    })
}

pub(crate) fn normalized_breadcrumbs(
    page: &ResolvedPage,
    site: &SiteView,
    navigation: &[NavSection],
    all_pages: &[ResolvedPage],
) -> Vec<BreadcrumbView> {
    if page.route == "/" {
        return Vec::new();
    }
    let mut crumbs = vec![BreadcrumbView::new(&site.title, "/")];
    if let Some(section) = current_section(navigation, &page.id)
        && let Some(first) = catalog::first_nav_item(section)
    {
        push_breadcrumb(
            &mut crumbs,
            BreadcrumbView::new(&section.label, &first.route),
        );
    }
    if let Some(rest) = page.id.strip_prefix("examples/")
        && let Some(example) = rest.split('/').next()
    {
        let parent_id = format!("examples/{example}/index");
        if let Some(parent) = all_pages.iter().find(|candidate| candidate.id == parent_id) {
            push_breadcrumb(
                &mut crumbs,
                BreadcrumbView::new(&parent.title, &parent.route),
            );
        }
    }
    for link in page.breadcrumbs.iter().filter(|link| link.route != "/") {
        let mut next = breadcrumb_from_link(link);
        if crumbs.last().is_some_and(|previous| {
            previous.href == next.href
                && !previous
                    .title
                    .trim()
                    .eq_ignore_ascii_case(next.title.trim())
        }) && let Some(group) = find_group_for_page(navigation, &link.title, &page.id)
            && let Some(item) = group.items.iter().find(|item| {
                crumbs
                    .last()
                    .is_none_or(|previous| item.route != previous.href)
            })
        {
            next.href = item.route.clone();
        }
        push_breadcrumb(&mut crumbs, next);
    }
    crumbs
}

fn find_group_for_page<'a>(
    navigation: &'a [NavSection],
    label: &str,
    page_id: &str,
) -> Option<&'a NavSection> {
    navigation.iter().find_map(|section| {
        if section.label.eq_ignore_ascii_case(label) && catalog::section_contains(section, page_id)
        {
            Some(section)
        } else {
            find_group_for_page(&section.children, label, page_id)
        }
    })
}

fn push_breadcrumb(crumbs: &mut Vec<BreadcrumbView>, next: BreadcrumbView) {
    let duplicate = crumbs.last().is_some_and(|previous| {
        previous.href == next.href
            || previous
                .title
                .trim()
                .eq_ignore_ascii_case(next.title.trim())
    });
    if !duplicate {
        crumbs.push(next);
    }
}

pub(crate) fn find_route_id<'a>(navigation: &'a [NavSection], route: &str) -> Option<&'a str> {
    for section in navigation {
        if let Some(item) = section.items.iter().find(|item| item.route == route) {
            return Some(item.id.as_str());
        }
        if let Some(id) = find_route_id(&section.children, route) {
            return Some(id);
        }
    }
    None
}

fn nav_leaf(item: &catalog::NavItem, current_id: Option<&str>) -> NavItemView {
    NavItemView {
        title: item.title.clone(),
        href: item.route.clone(),
        class_name: if current_id == Some(item.id.as_str()) {
            "nav-link nav-child is-current".into()
        } else {
            "nav-link nav-child".into()
        },
    }
}

fn nav_item_owns_page(item_id: &str, current_id: &str) -> bool {
    if current_id == item_id {
        return true;
    }
    let Some(example) = item_id.strip_prefix("examples/") else {
        return false;
    };
    let Some(slug) = example.strip_suffix("/index") else {
        return false;
    };
    current_id
        .strip_prefix("examples/")
        .is_some_and(|rest| rest == slug || rest.starts_with(&format!("{slug}/")))
}

fn index_dir(id: &str) -> Option<&str> {
    id.strip_suffix("/index")
}

fn section_root_dir(items: &[catalog::NavItem]) -> Option<&str> {
    items
        .iter()
        .filter_map(|item| index_dir(&item.id))
        .min_by_key(|dir| dir.len())
}

fn is_under_dir(dir: &str, id: &str) -> bool {
    id == dir || id.starts_with(&format!("{dir}/"))
}

fn is_fold_index(id: &str, root: Option<&str>) -> bool {
    let Some(dir) = index_dir(id) else {
        return false;
    };
    root.is_some_and(|root| dir.len() > root.len() && dir.starts_with(&format!("{root}/")))
}

struct BuildingGroup {
    index: catalog::NavItem,
    dir: String,
    items: Vec<catalog::NavItem>,
    children: Vec<NavGroupView>,
}

enum ForestRow {
    Item(NavItemView),
    Group(NavGroupView),
}

fn overview_item(href: &str, current: bool) -> NavItemView {
    NavItemView {
        title: "Overview".into(),
        href: href.into(),
        class_name: if current {
            "nav-link nav-child is-current".into()
        } else {
            "nav-link nav-child".into()
        },
    }
}

fn prepend_overview(items: &mut Vec<NavItemView>, href: &str, current: bool) {
    if href.is_empty() {
        return;
    }
    if items
        .first()
        .is_some_and(|item| item.title == "Overview" && item.href == href)
    {
        return;
    }
    items.insert(0, overview_item(href, current));
}

fn is_landing_overview(item: &NavItemView, href: &str) -> bool {
    item.title == "Overview" && item.href == href
}

fn flush_building(building: BuildingGroup, current_id: Option<&str>) -> ForestRow {
    if building.items.is_empty() && building.children.is_empty() {
        return ForestRow::Item(nav_leaf(&building.index, current_id));
    }
    let mut items: Vec<NavItemView> = building
        .items
        .iter()
        .map(|item| nav_leaf(item, current_id))
        .collect();
    let landing_current = current_id.is_some_and(|id| nav_item_owns_page(&building.index.id, id));
    prepend_overview(&mut items, &building.index.route, landing_current);
    let open = current_id.is_some_and(|id| {
        is_under_dir(&building.dir, id)
            || nav_item_owns_page(&building.index.id, id)
            || building
                .items
                .iter()
                .any(|item| nav_item_owns_page(&item.id, id))
    }) || building.children.iter().any(|child| child.open);
    ForestRow::Group(NavGroupView {
        title: building.index.title,
        href: building.index.route,
        open,
        items,
        children: building.children,
    })
}

fn attach_row(
    row: ForestRow,
    stack: &mut [BuildingGroup],
    rows: &mut Vec<ForestRow>,
    current_id: Option<&str>,
) {
    if let Some(parent) = stack.last_mut() {
        match row {
            ForestRow::Item(item) => parent.children.push(leaf_group(item)),
            ForestRow::Group(group) => parent.children.push(group),
        }
        return;
    }
    let _ = current_id;
    rows.push(row);
}

fn forest_from_items(items: &[catalog::NavItem], current_id: Option<&str>) -> Vec<ForestRow> {
    let root = section_root_dir(items);
    let mut rows = Vec::new();
    let mut stack: Vec<BuildingGroup> = Vec::new();
    for item in items {
        while stack
            .last()
            .is_some_and(|top| !is_under_dir(&top.dir, &item.id))
        {
            let finished = stack.pop().expect("stack");
            attach_row(
                flush_building(finished, current_id),
                &mut stack,
                &mut rows,
                current_id,
            );
        }
        if is_fold_index(&item.id, root) {
            stack.push(BuildingGroup {
                index: item.clone(),
                dir: index_dir(&item.id).expect("fold index").to_string(),
                items: Vec::new(),
                children: Vec::new(),
            });
        } else if let Some(top) = stack.last_mut() {
            top.items.push(item.clone());
        } else {
            rows.push(ForestRow::Item(nav_leaf(item, current_id)));
        }
    }
    while let Some(finished) = stack.pop() {
        attach_row(
            flush_building(finished, current_id),
            &mut stack,
            &mut rows,
            current_id,
        );
    }
    rows
}

fn leaf_group(item: NavItemView) -> NavGroupView {
    let open = item.class_name.contains("is-current");
    NavGroupView {
        title: item.title,
        href: item.href,
        open,
        items: Vec::new(),
        children: Vec::new(),
    }
}

fn is_group_root_index(id: &str, items: &[catalog::NavItem]) -> bool {
    if id == "index" {
        return items.iter().any(|item| item.id == "index");
    }
    let Some(dir) = index_dir(id) else {
        return false;
    };
    section_root_dir(items).is_some_and(|root| dir == root)
}

fn root_index_href(items: &[catalog::NavItem], has_siblings: bool) -> Option<String> {
    if items.len() <= 1 && !has_siblings {
        return None;
    }
    let first = items.first()?;
    is_group_root_index(&first.id, items).then(|| first.route.clone())
}

fn take_landing_href(rows: &mut Vec<ForestRow>, landing: Option<String>) -> String {
    let Some(href) = landing else {
        return String::new();
    };
    let matches = match rows.first() {
        Some(ForestRow::Item(item)) => item.href == href,
        Some(ForestRow::Group(child)) => {
            child.items.is_empty() && child.children.is_empty() && child.href == href
        }
        None => false,
    };
    if matches {
        rows.remove(0);
        href
    } else {
        String::new()
    }
}

fn rows_to_group(
    label: &str,
    mut rows: Vec<ForestRow>,
    extra_children: Vec<NavGroupView>,
    open: bool,
    section_items: &[catalog::NavItem],
    current_id: Option<&str>,
) -> Option<NavGroupView> {
    let landing = root_index_href(section_items, !extra_children.is_empty() || rows.len() > 1);
    let landing_current = landing.as_ref().is_some_and(|href| {
        section_items.first().is_some_and(|item| {
            item.route == *href && current_id.is_some_and(|id| nav_item_owns_page(&item.id, id))
        })
    });
    let has_group =
        rows.iter().any(|row| matches!(row, ForestRow::Group(_))) || !extra_children.is_empty();
    if !has_group {
        let mut items: Vec<NavItemView> = rows
            .into_iter()
            .map(|row| match row {
                ForestRow::Item(item) => item,
                ForestRow::Group(_) => unreachable!(),
            })
            .collect();
        if items.is_empty() && extra_children.is_empty() {
            return None;
        }
        let href = if let Some(href) =
            landing.filter(|href| items.first().is_some_and(|item| item.href == *href))
        {
            items.remove(0);
            href
        } else {
            String::new()
        };
        prepend_overview(&mut items, &href, landing_current);
        return Some(flatten_group_depth(NavGroupView {
            title: label.into(),
            href,
            open: open || extra_children.iter().any(|child| child.open),
            items,
            children: extra_children,
        }));
    }
    let href = take_landing_href(&mut rows, landing);
    let mut children = Vec::new();
    for row in rows {
        match row {
            ForestRow::Item(item) => children.push(leaf_group(item)),
            ForestRow::Group(group) => children.push(group),
        }
    }
    children.extend(extra_children);
    let mut items = Vec::new();
    prepend_overview(&mut items, &href, landing_current);
    Some(flatten_group_depth(NavGroupView {
        title: label.into(),
        href,
        open: open
            || children.iter().any(|child| child.open)
            || items
                .iter()
                .any(|item| item.class_name.contains("is-current")),
        items,
        children,
    }))
}

fn nav_group_view(section: &NavSection, current_id: Option<&str>) -> Option<NavGroupView> {
    let rows = forest_from_items(&section.items, current_id);
    let extra_children = section
        .children
        .iter()
        .filter_map(|child| nav_group_view(child, current_id))
        .collect();
    let open = current_id.is_some_and(|id| {
        catalog::section_contains(section, id)
            || section
                .items
                .iter()
                .any(|item| nav_item_owns_page(&item.id, id))
    });
    rows_to_group(
        &section.label,
        rows,
        extra_children,
        open,
        &section.items,
        current_id,
    )
}

fn flatten_group_depth(mut group: NavGroupView) -> NavGroupView {
    let mut next = Vec::new();
    for child in group.children {
        let child = flatten_group_depth(child);
        let grandchildren = child.children;
        next.push(NavGroupView {
            title: child.title,
            href: child.href,
            open: child.open || grandchildren.iter().any(|grand| grand.open),
            items: child.items,
            children: Vec::new(),
        });
        next.extend(grandchildren);
    }
    group.children = next;
    group
}

fn selected_example_slug(page_id: &str) -> Option<&str> {
    let rest = page_id.strip_prefix("examples/")?;
    let slug = rest.split('/').next()?;
    if slug.is_empty() || slug == "index" {
        None
    } else {
        Some(slug)
    }
}

fn example_source_prefix(slug: &str) -> String {
    format!("examples/{slug}/source/")
}

pub(crate) fn attach_example_source_tree(
    sidebar: &mut Vec<NavGroupView>,
    current_id: Option<&str>,
    pages: &[ResolvedPage],
) {
    let Some(current_id) = current_id else {
        return;
    };
    let Some(slug) = selected_example_slug(current_id) else {
        return;
    };
    let example_href = format!("/examples/{slug}/");
    let prefix = example_source_prefix(slug);
    let mut source_pages: Vec<&ResolvedPage> = pages
        .iter()
        .filter(|page| page.id.starts_with(&prefix))
        .collect();
    if source_pages.is_empty() {
        return;
    }
    source_pages.sort_by(|left, right| left.id.cmp(&right.id));
    let Some(group_index) = sidebar
        .iter()
        .position(|group| group.items.iter().any(|item| item.href == example_href))
    else {
        return;
    };
    let group = sidebar.remove(group_index);
    let source_items: Vec<NavItemView> = source_pages
        .iter()
        .map(|page| {
            let current = current_id == page.id.as_str();
            NavItemView {
                title: page.title.clone(),
                href: page.route.clone(),
                class_name: if current {
                    "nav-link nav-child nav-source is-current".into()
                } else {
                    "nav-link nav-child nav-source".into()
                },
            }
        })
        .collect();
    let mut replacement = Vec::new();
    if !group.href.is_empty()
        && group
            .items
            .first()
            .is_none_or(|item| item.href != group.href || is_landing_overview(item, &group.href))
    {
        replacement.push(NavGroupView {
            title: group.title.clone(),
            href: group.href.clone(),
            open: false,
            items: Vec::new(),
            children: Vec::new(),
        });
    }
    for item in group.items {
        if is_landing_overview(&item, &group.href) {
            continue;
        }
        let selected = item.href == example_href;
        replacement.push(NavGroupView {
            title: item.title,
            href: item.href,
            open: false,
            items: Vec::new(),
            children: Vec::new(),
        });
        if selected {
            replacement.push(NavGroupView {
                title: "Source".into(),
                href: String::new(),
                open: true,
                items: source_items.clone(),
                children: Vec::new(),
            });
        }
    }
    sidebar.splice(group_index..group_index, replacement);
}

pub(crate) fn outline_view(heading: &PageHeading) -> OutlineView {
    OutlineView {
        id: heading.id.clone(),
        title: heading.text.clone(),
        level: heading.level.to_string(),
    }
}

fn breadcrumb_from_link(link: &NavLink) -> BreadcrumbView {
    BreadcrumbView::new(&link.title, &link.route)
}

fn nav_from_link(link: &NavLink) -> NavItemView {
    NavItemView {
        title: link.title.clone(),
        href: link.route.clone(),
        class_name: String::new(),
    }
}

pub(crate) fn optional_link(link: Option<&NavLink>) -> NavItemView {
    match link {
        Some(link) => nav_from_link(link),
        None => NavItemView {
            title: String::new(),
            href: String::new(),
            class_name: String::new(),
        },
    }
}
