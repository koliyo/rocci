#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavCommand {
    Back,
    Forward,
    Home,
    Reload,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IpcMessage {
    Nav(NavCommand),
    Reveal(String),
    CopySource(String),
    LiveReload(bool),
}

impl NavCommand {
    pub fn parse(message: &str) -> Option<Self> {
        match message.trim() {
            "back" => Some(Self::Back),
            "forward" => Some(Self::Forward),
            "home" => Some(Self::Home),
            "reload" => Some(Self::Reload),
            _ => None,
        }
    }
}

impl IpcMessage {
    pub fn parse(message: &str) -> Option<Self> {
        let message = message.trim();
        if let Some(path) = message.strip_prefix("reveal:") {
            return Some(Self::Reveal(path.to_string()));
        }
        if let Some(path) = message.strip_prefix("copy-source:") {
            return Some(Self::CopySource(path.to_string()));
        }
        if let Some(value) = message.strip_prefix("live-reload:") {
            return Some(Self::LiveReload(value == "1"));
        }
        NavCommand::parse(message).map(Self::Nav)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Intent {
    Push,
    Back,
    Forward,
    Home,
}

#[derive(Debug)]
pub struct NavHistory {
    home: String,
    stack: Vec<String>,
    index: usize,
    intent: Intent,
}

impl NavHistory {
    pub fn new(home: impl Into<String>) -> Self {
        Self {
            home: normalize_url(&home.into()),
            stack: Vec::new(),
            index: 0,
            intent: Intent::Push,
        }
    }

    pub fn home(&self) -> &str {
        &self.home
    }

    pub fn current(&self) -> Option<&str> {
        self.stack.get(self.index).map(String::as_str)
    }

    pub fn can_back(&self) -> bool {
        self.index > 0
    }

    pub fn can_forward(&self) -> bool {
        !self.stack.is_empty() && self.index + 1 < self.stack.len()
    }

    pub fn display_path(&self) -> String {
        display_path(self.current().unwrap_or(&self.home))
    }

    pub fn request_back(&mut self) -> bool {
        if !self.can_back() {
            return false;
        }
        self.intent = Intent::Back;
        true
    }

    pub fn request_forward(&mut self) -> bool {
        if !self.can_forward() {
            return false;
        }
        self.intent = Intent::Forward;
        true
    }

    pub fn request_home(&mut self) {
        self.intent = Intent::Home;
    }

    pub fn commit(&mut self, url: &str) {
        let url = normalize_url(url);
        if url.is_empty() || url == "about:blank" {
            return;
        }
        let intent = std::mem::replace(&mut self.intent, Intent::Push);
        match intent {
            Intent::Back => self.commit_relative(&url, -1),
            Intent::Forward => self.commit_relative(&url, 1),
            Intent::Home => {
                self.stack.clear();
                self.stack.push(if url.is_empty() {
                    self.home.clone()
                } else {
                    url
                });
                self.index = 0;
            }
            Intent::Push => {
                if self.stack.get(self.index) == Some(&url) {
                    return;
                }
                self.push_url(url);
            }
        }
    }

    fn commit_relative(&mut self, url: &str, delta: isize) {
        let next = self.index.checked_add_signed(delta);
        if let Some(index) = next
            && index < self.stack.len()
            && self.stack[index] == url
        {
            self.index = index;
            return;
        }
        if let Some(index) = self.stack.iter().position(|entry| entry == url) {
            self.index = index;
            return;
        }
        self.push_url(url.to_string());
    }

    fn push_url(&mut self, url: String) {
        if self.stack.is_empty() {
            self.stack.push(url);
            self.index = 0;
            return;
        }
        self.stack.truncate(self.index + 1);
        self.stack.push(url);
        self.index += 1;
    }
}

pub fn normalize_url(url: &str) -> String {
    url.trim().trim_end_matches('#').to_string()
}

pub fn display_path(url: &str) -> String {
    let rest = url.split_once("://").map(|(_, rest)| rest).unwrap_or(url);
    match rest.find('/') {
        Some(index) => rest[index..].to_string(),
        None if rest.is_empty() => "/".to_string(),
        None => "/".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GUIDE: &str = "http://127.0.0.1:8000/guides/rocdown/";
    const INTERACTIVE: &str = "http://127.0.0.1:8000/guides/rocdown-interactive/";
    const ABOUT: &str = "http://127.0.0.1:8000/about/";

    #[test]
    fn parse_known_commands() {
        assert_eq!(NavCommand::parse("back"), Some(NavCommand::Back));
        assert_eq!(NavCommand::parse(" forward\n"), Some(NavCommand::Forward));
        assert_eq!(NavCommand::parse("home"), Some(NavCommand::Home));
        assert_eq!(NavCommand::parse("reload"), Some(NavCommand::Reload));
        assert_eq!(NavCommand::parse("nope"), None);
        assert_eq!(
            IpcMessage::parse("reveal:architecture/overview.md"),
            Some(IpcMessage::Reveal("architecture/overview.md".into()))
        );
        assert_eq!(
            IpcMessage::parse("copy-source:guides/rocdown.rocdown"),
            Some(IpcMessage::CopySource("guides/rocdown.rocdown".into()))
        );
        assert_eq!(
            IpcMessage::parse("live-reload:0"),
            Some(IpcMessage::LiveReload(false))
        );
        assert_eq!(
            IpcMessage::parse("live-reload:1"),
            Some(IpcMessage::LiveReload(true))
        );
        assert_eq!(
            IpcMessage::parse("home"),
            Some(IpcMessage::Nav(NavCommand::Home))
        );
    }

    #[test]
    fn first_load_sets_path_without_history() {
        let mut history = NavHistory::new(GUIDE);
        history.commit(GUIDE);
        assert!(!history.can_back());
        assert!(!history.can_forward());
        assert_eq!(history.current(), Some(GUIDE));
        assert_eq!(history.display_path(), "/guides/rocdown/");
    }

    #[test]
    fn sibling_routes_are_distinct_entries() {
        let mut history = NavHistory::new(GUIDE);
        history.commit(GUIDE);
        history.commit(INTERACTIVE);
        assert!(history.can_back());
        assert!(!history.can_forward());
        assert_eq!(history.display_path(), "/guides/rocdown-interactive/");
        assert_ne!(display_path(GUIDE), display_path(INTERACTIVE));
    }

    #[test]
    fn back_and_forward_move_through_stack() {
        let mut history = NavHistory::new(GUIDE);
        history.commit(GUIDE);
        history.commit(INTERACTIVE);
        assert!(history.request_back());
        history.commit(GUIDE);
        assert!(!history.can_back());
        assert!(history.can_forward());
        assert_eq!(history.current(), Some(GUIDE));
        assert!(history.request_forward());
        history.commit(INTERACTIVE);
        assert!(history.can_back());
        assert!(!history.can_forward());
        assert_eq!(history.current(), Some(INTERACTIVE));
    }

    #[test]
    fn further_navigation_truncates_forward() {
        let mut history = NavHistory::new(GUIDE);
        history.commit(GUIDE);
        history.commit(INTERACTIVE);
        history.request_back();
        history.commit(GUIDE);
        history.commit(ABOUT);
        assert!(history.can_back());
        assert!(!history.can_forward());
        assert_eq!(history.current(), Some(ABOUT));
        assert!(!history.request_forward());
        history.request_back();
        history.commit(GUIDE);
        assert_eq!(history.current(), Some(GUIDE));
    }

    #[test]
    fn home_resets_to_home() {
        let mut history = NavHistory::new(GUIDE);
        history.commit(GUIDE);
        history.commit(INTERACTIVE);
        history.request_home();
        history.commit(GUIDE);
        assert!(!history.can_back());
        assert!(!history.can_forward());
        assert_eq!(history.current(), Some(GUIDE));
        history.commit(INTERACTIVE);
        assert!(history.can_back());
    }

    #[test]
    fn same_url_reload_does_not_push() {
        let mut history = NavHistory::new(GUIDE);
        history.commit(GUIDE);
        history.commit(GUIDE);
        assert!(!history.can_back());
        history.commit(INTERACTIVE);
        history.commit(INTERACTIVE);
        assert!(history.can_back());
        assert!(!history.can_forward());
        history.request_back();
        history.commit(GUIDE);
        assert!(!history.can_back());
        assert!(history.can_forward());
    }

    #[test]
    fn skips_blank_and_ignores_back_without_history() {
        let mut history = NavHistory::new(GUIDE);
        history.commit("about:blank");
        history.commit("");
        assert_eq!(history.current(), None);
        assert!(!history.request_back());
        history.commit(GUIDE);
        assert!(!history.request_back());
        assert!(!history.request_forward());
    }

    #[test]
    fn display_path_strips_origin() {
        assert_eq!(
            display_path("http://127.0.0.1:9377/guides/rocdown-interactive/"),
            "/guides/rocdown-interactive/"
        );
        assert_eq!(display_path("https://example.test"), "/");
    }
}
