mod mode_manager;

use crate::homescreen::mode_manager::ModeManager;

use dirs::state_dir;

use iced::Element;
use iced::Task;
use iced::clipboard;
use iced::keyboard::{Key, Modifiers};
use iced::widget::{Column, text_editor};
use iced::window;

use serde::{Deserialize, Serialize};

use std::error;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub enum Message {
    Keypress(Key, Modifiers),
    EditorActivate(text_editor::Action, text_editor::Id),
}

#[derive(Debug)]
enum SavestateError {
    Io(io::Error),
    NoParent,
}

impl fmt::Display for SavestateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SavestateError::Io(err) => write!(f, "IO Error: {}", err),
            SavestateError::NoParent => write!(f, "Parent dir does not exist"),
        }
    }
}

impl error::Error for SavestateError {
    fn cause(&self) -> Option<&dyn error::Error> {
        match self {
            SavestateError::Io(err) => Some(err),
            SavestateError::NoParent => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Savefile {
    log_entries: Vec<String>,
}

#[derive(Debug)]
pub struct Counter {
    state_file: Option<PathBuf>,
    edit_entries: Vec<(text_editor::Id, text_editor::Content)>,
    mode_mgr: ModeManager,
}

impl Default for Counter {
    fn default() -> Self {
        let state_file = match state_dir() {
            Some(sd) => Some(sd.join("polartales/state.json")),
            None => None,
        };
        let log_entries = match &state_file {
            Some(f) => Counter::read_logs_from_json(f),
            None => Vec::new(),
        };

        let edit_entries = log_entries
            .iter()
            .map(|e| {
                let mut e = text_editor::Content::with_text(&e);
                e.perform(text_editor::Action::Move(text_editor::Motion::DocumentEnd));
                e
            })
            .map(|e| (text_editor::Id::unique(), e))
            .collect();

        Counter {
            state_file,
            mode_mgr: ModeManager::new(),
            edit_entries,
        }
    }
}

impl Counter {
    fn read_logs_from_json(p: impl AsRef<Path>) -> Vec<String> {
        let Ok(text) = fs::read_to_string(p) else {
            return Vec::new();
        };

        match serde_json::from_str::<Savefile>(&text) {
            Ok(state) => state.log_entries,
            Err(_) => Vec::new(),
        }
    }

    fn write_logs_to_json(p: impl AsRef<Path>, contents: &str) -> Result<(), SavestateError> {
        let Some(parent) = p.as_ref().parent() else {
            return Err(SavestateError::NoParent);
        };
        fs::create_dir_all(parent).map_err(SavestateError::Io)?;
        let newline_terminated_contents = format!("{contents}\n");
        fs::write(p, newline_terminated_contents).map_err(SavestateError::Io)?;
        Ok(())
    }

    fn add_note(&mut self) -> Task<Message> {
        let id = text_editor::Id::unique();
        self.edit_entries
            .push((id.clone(), text_editor::Content::new()));
        text_editor::focus(id)
    }

    fn save_and_exit(&self) -> Task<Message> {
        let all_notes: Vec<String> = self
            .edit_entries
            .iter()
            .map(|(_, content)| content.text())
            .collect();
        let text_notes = all_notes.join("\n");
        let state = Savefile {
            log_entries: all_notes,
        };
        if let Ok(json_notes) = serde_json::to_string_pretty(&state) {
            if let Some(state_file) = &self.state_file {
                let _ = Counter::write_logs_to_json(state_file, &json_notes);
            }
        } else {
            println!("{state:?}");
        };
        // iced::exit() is tempting to return without closing the window first,
        // but this results in a segfault in iced on some platforms, so closing
        // the window first is cleaner, at least for now.
        // https://github.com/iced-rs/iced/issues/2983
        clipboard::write(text_notes).chain(
            window::get_latest()
                .and_then(window::close)
                .chain(iced::exit()),
        )
    }

    fn focus_entry(&self, entry_idx: usize) -> Option<Task<Message>> {
        match self.edit_entries.get(entry_idx) {
            Some((id, _)) => Some(text_editor::focus(id.clone())),
            None => None,
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Keypress(k, m) => {
                // well, this is awkward
                let mut mode_mgr_tmp = self.mode_mgr;
                let task = mode_mgr_tmp.handle_keypress(k, m, self);
                self.mode_mgr = mode_mgr_tmp;
                task
            },
            Message::EditorActivate(action, target_id) => {
                match self
                    .edit_entries
                    .iter_mut()
                    .find(|(id, _)| id == &target_id)
                {
                    Some((_, content)) => content.perform(action),
                    None => {}
                }
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let mut entries = Column::new();
        for (id, content) in &self.edit_entries {
            entries = entries.push(
                text_editor(content)
                    .placeholder("notes")
                    .id(id.clone())
                    .on_action(move |a| Message::EditorActivate(a, id.clone())),
            );
        }

        entries.into()
    }
}
