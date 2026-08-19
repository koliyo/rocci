use std::io::{Write, stdout};

use anyhow::{Result, bail};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, ClearType},
};

use rocci_browser::{Host, Opened, Picker, PickerAction, PickerOutcome, PickerStage};

pub fn run(host: &mut Host, no_window: bool, json: bool) -> Result<Option<Opened>> {
    let targets = host.probe_targets()?;
    if targets.is_empty() {
        bail!("no claimed targets; add a directory with `rocci-browser add`");
    }
    let mut picker = Picker::new(targets);
    terminal::enable_raw_mode()?;
    let result = (|| -> Result<Option<Opened>> {
        loop {
            draw(&picker)?;
            let Event::Key(key) = event::read()? else {
                continue;
            };
            if key.kind != KeyEventKind::Press {
                continue;
            }
            let action = match key.code {
                KeyCode::Esc => PickerAction::Escape,
                KeyCode::Enter => PickerAction::Enter,
                KeyCode::Backspace => PickerAction::Backspace,
                KeyCode::Up => PickerAction::Move(-1),
                KeyCode::Down => PickerAction::Move(1),
                KeyCode::Tab if key.modifiers.contains(KeyModifiers::SHIFT) => {
                    PickerAction::ShiftTab
                }
                KeyCode::BackTab => PickerAction::ShiftTab,
                KeyCode::Tab => PickerAction::Tab,
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    PickerAction::Escape
                }
                KeyCode::Char(ch) => PickerAction::Insert(ch),
                _ => continue,
            };
            match picker.apply(action) {
                PickerOutcome::Continue => {}
                PickerOutcome::Cancel => return Ok(None),
                PickerOutcome::NeedDocuments { adapter_id, root } => {
                    let (documents, reason) = host.documents_or_reason(&adapter_id, &root)?;
                    picker.enter_documents(documents, reason);
                }
                PickerOutcome::OpenTarget { adapter_id, root } => {
                    return Ok(Some(host.open_target(&adapter_id, &root, None)?));
                }
                PickerOutcome::OpenDocument {
                    adapter_id,
                    root,
                    document,
                } => {
                    return Ok(Some(host.open_target(
                        &adapter_id,
                        &root,
                        Some(&document),
                    )?));
                }
            }
        }
    })();
    terminal::disable_raw_mode()?;
    execute!(stdout(), terminal::Clear(ClearType::All), ResetColor)?;
    let opened = result?;
    if let Some(opened) = &opened {
        super::emit_open(opened, no_window, json)?;
    }
    Ok(opened)
}

fn draw(picker: &Picker) -> Result<()> {
    let mut out = stdout();
    execute!(
        out,
        terminal::Clear(ClearType::All),
        crossterm::cursor::MoveTo(0, 0)
    )?;
    let stage = match picker.stage {
        PickerStage::Targets => "targets",
        PickerStage::Documents => "documents",
    };
    writeln!(
        out,
        "rocci-browser  {stage}  Tab lists documents  Enter opens  Esc quits"
    )?;
    writeln!(out, "> {}", picker.query)?;
    if let Some(reason) = &picker.empty_reason {
        writeln!(out, "{reason}")?;
    }
    match picker.stage {
        PickerStage::Targets => {
            for (index, (_, target)) in picker.visible_targets().into_iter().take(16).enumerate() {
                write_row(
                    &mut out,
                    index == picker.selected,
                    &format!(
                        "{}  {}  [{}] {}",
                        target.id, target.path, target.adapter_id, target.label
                    ),
                )?;
            }
        }
        PickerStage::Documents => {
            for (index, (_, document)) in
                picker.visible_documents().into_iter().take(16).enumerate()
            {
                write_row(
                    &mut out,
                    index == picker.selected,
                    &format!("{}  {}", document.title, document.path),
                )?;
            }
        }
    }
    out.flush()?;
    Ok(())
}

fn write_row(out: &mut impl Write, selected: bool, text: &str) -> Result<()> {
    if selected {
        execute!(
            out,
            SetBackgroundColor(Color::DarkBlue),
            SetForegroundColor(Color::White),
            Print(format!("{text}\n")),
            ResetColor
        )?;
    } else {
        writeln!(out, "{text}")?;
    }
    Ok(())
}
