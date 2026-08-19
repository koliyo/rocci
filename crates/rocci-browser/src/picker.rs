use crate::{
    fuzzy::{ScoreFields, score_entry},
    host::Target,
    protocol::Document,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PickerStage {
    Targets,
    Documents,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PickerAction {
    Insert(char),
    Backspace,
    Enter,
    Tab,
    ShiftTab,
    Escape,
    Move(isize),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PickerOutcome {
    Continue,
    OpenTarget {
        adapter_id: String,
        root: String,
    },
    OpenDocument {
        adapter_id: String,
        root: String,
        document: String,
    },
    Cancel,
    NeedDocuments {
        adapter_id: String,
        root: String,
    },
}

#[derive(Clone, Debug)]
pub struct Picker {
    pub query: String,
    pub stage: PickerStage,
    pub selected: usize,
    pub targets: Vec<Target>,
    pub documents: Vec<Document>,
    pub empty_reason: Option<String>,
    pub target_query: String,
    highlighted_target: Option<Target>,
}

impl Picker {
    pub fn new(targets: Vec<Target>) -> Self {
        Self {
            query: String::new(),
            stage: PickerStage::Targets,
            selected: 0,
            targets,
            documents: Vec::new(),
            empty_reason: None,
            target_query: String::new(),
            highlighted_target: None,
        }
    }

    pub fn visible_targets(&self) -> Vec<(f64, &Target)> {
        rank_targets(&self.targets, &self.query)
    }

    pub fn visible_documents(&self) -> Vec<(f64, &Document)> {
        rank_documents(&self.documents, &self.query)
    }

    pub fn highlighted_target(&self) -> Option<&Target> {
        let rows = self.visible_targets();
        rows.get(self.selected).map(|(_, target)| *target)
    }

    pub fn apply(&mut self, action: PickerAction) -> PickerOutcome {
        match action {
            PickerAction::Insert(ch) => {
                self.query.push(ch);
                self.selected = 0;
                PickerOutcome::Continue
            }
            PickerAction::Backspace => {
                self.query.pop();
                self.selected = 0;
                PickerOutcome::Continue
            }
            PickerAction::Move(delta) => {
                let len = match self.stage {
                    PickerStage::Targets => self.visible_targets().len(),
                    PickerStage::Documents => self.visible_documents().len(),
                };
                if len == 0 {
                    return PickerOutcome::Continue;
                }
                let next = self.selected as isize + delta;
                self.selected = next.rem_euclid(len as isize) as usize;
                PickerOutcome::Continue
            }
            PickerAction::Escape => match self.stage {
                PickerStage::Documents => {
                    self.back_to_targets();
                    PickerOutcome::Continue
                }
                PickerStage::Targets => PickerOutcome::Cancel,
            },
            PickerAction::ShiftTab => {
                if self.stage == PickerStage::Documents {
                    self.back_to_targets();
                }
                PickerOutcome::Continue
            }
            PickerAction::Tab => {
                if self.stage != PickerStage::Targets {
                    return PickerOutcome::Continue;
                }
                let Some(target) = self.highlighted_target().cloned() else {
                    return PickerOutcome::Continue;
                };
                self.highlighted_target = Some(target.clone());
                PickerOutcome::NeedDocuments {
                    adapter_id: target.adapter_id,
                    root: target.path,
                }
            }
            PickerAction::Enter => match self.stage {
                PickerStage::Targets => {
                    let Some(target) = self.highlighted_target().cloned() else {
                        return PickerOutcome::Continue;
                    };
                    PickerOutcome::OpenTarget {
                        adapter_id: target.adapter_id,
                        root: target.path,
                    }
                }
                PickerStage::Documents => {
                    let rows = self.visible_documents();
                    let Some((_, document)) = rows.get(self.selected) else {
                        return PickerOutcome::Continue;
                    };
                    let Some(target) = self.highlighted_target.clone() else {
                        return PickerOutcome::Continue;
                    };
                    PickerOutcome::OpenDocument {
                        adapter_id: target.adapter_id,
                        root: target.path,
                        document: document.id.clone(),
                    }
                }
            },
        }
    }

    pub fn enter_documents(&mut self, documents: Vec<Document>, reason: Option<String>) {
        if documents.is_empty() {
            self.empty_reason = reason.or_else(|| Some("adapter returned no documents".into()));
            self.highlighted_target = None;
            return;
        }
        self.target_query = self.query.clone();
        self.documents = documents;
        self.query.clear();
        self.selected = 0;
        self.empty_reason = None;
        self.stage = PickerStage::Documents;
    }

    fn back_to_targets(&mut self) {
        self.stage = PickerStage::Targets;
        self.query = self.target_query.clone();
        self.documents.clear();
        self.selected = 0;
        self.highlighted_target = None;
        self.empty_reason = None;
    }
}

pub fn rank_targets<'a>(targets: &'a [Target], query: &str) -> Vec<(f64, &'a Target)> {
    let mut rows: Vec<(f64, &Target)> = targets
        .iter()
        .filter_map(|target| {
            let score = score_entry(
                query,
                ScoreFields {
                    title: &target.id,
                    path: &target.path,
                    description: Some(&target.label),
                    url: None,
                },
            );
            (score >= 0.0).then_some((score, target))
        })
        .collect();
    rows.sort_by(|a, b| {
        b.0.total_cmp(&a.0)
            .then_with(|| a.1.adapter_id.cmp(&b.1.adapter_id))
            .then_with(|| a.1.id.cmp(&b.1.id))
    });
    rows
}

pub fn rank_documents<'a>(documents: &'a [Document], query: &str) -> Vec<(f64, &'a Document)> {
    let mut rows: Vec<(f64, &Document)> = documents
        .iter()
        .filter_map(|document| {
            let score = score_entry(
                query,
                ScoreFields {
                    title: &document.title,
                    path: &document.path,
                    description: None,
                    url: document.route.as_deref(),
                },
            );
            (score >= 0.0).then_some((score, document))
        })
        .collect();
    rows.sort_by(|a, b| {
        b.0.total_cmp(&a.0)
            .then_with(|| a.1.title.cmp(&b.1.title))
            .then_with(|| a.1.id.cmp(&b.1.id))
    });
    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target(id: &str, adapter: &str) -> Target {
        Target {
            id: id.into(),
            path: format!("/tmp/{id}"),
            adapter_id: adapter.into(),
            label: adapter.into(),
            detail: None,
        }
    }

    #[test]
    fn enter_opens_target_tab_requests_documents() {
        let mut picker = Picker::new(vec![target("fixture", "a")]);
        assert_eq!(
            picker.apply(PickerAction::Enter),
            PickerOutcome::OpenTarget {
                adapter_id: "a".into(),
                root: "/tmp/fixture".into(),
            }
        );
        let mut picker = Picker::new(vec![target("fixture", "a")]);
        assert_eq!(
            picker.apply(PickerAction::Tab),
            PickerOutcome::NeedDocuments {
                adapter_id: "a".into(),
                root: "/tmp/fixture".into(),
            }
        );
        picker.enter_documents(
            vec![Document {
                id: "about".into(),
                title: "About".into(),
                path: "about.html".into(),
                route: Some("/about".into()),
            }],
            None,
        );
        assert_eq!(picker.stage, PickerStage::Documents);
        assert_eq!(
            picker.apply(PickerAction::Enter),
            PickerOutcome::OpenDocument {
                adapter_id: "a".into(),
                root: "/tmp/fixture".into(),
                document: "about".into(),
            }
        );
    }

    #[test]
    fn empty_document_list_stays_on_targets() {
        let mut picker = Picker::new(vec![target("fixture", "a")]);
        let _ = picker.apply(PickerAction::Tab);
        picker.enter_documents(Vec::new(), Some("none".into()));
        assert_eq!(picker.stage, PickerStage::Targets);
        assert_eq!(picker.empty_reason.as_deref(), Some("none"));
    }
}
