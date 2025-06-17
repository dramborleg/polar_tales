use crate::homescreen::Message;
use crate::savestate::Savefile;

use dirs::state_dir;

use iced::Task;
use iced::clipboard;
use iced::widget::{Column, text_editor};
use iced::window;

use std::path::PathBuf;

#[derive(Debug)]
pub struct NoteEditors {
    state_file: Option<PathBuf>,
    entries: Vec<(text_editor::Id, text_editor::Content)>,
}

impl NoteEditors {
    pub fn new() -> NoteEditors {
        let state_file = match state_dir() {
            Some(sd) => Some(sd.join("polartales/state.json")),
            None => None,
        };

        let log_entries = match &state_file {
            Some(f) => {
                match Savefile::read_from_json(f) {
                    Some(s) => s.log_entries,
                    None => Vec::new(),
                }
            },
            None => Vec::new(),
        };

        let entries = log_entries
            .iter()
            .map(|e| {
                let mut e = text_editor::Content::with_text(&e);
                e.perform(text_editor::Action::Move(text_editor::Motion::DocumentEnd));
                e
            })
            .map(|e| (text_editor::Id::unique(), e))
            .collect();
        NoteEditors {
            state_file,
            entries,
        }
    }

    pub fn add_note(&mut self) -> Task<Message> {
        let id = text_editor::Id::unique();
        self.entries.push((id.clone(), text_editor::Content::new()));
        text_editor::focus(id)
    }

    pub fn focus_entry(&self, entry_idx: usize) -> Option<Task<Message>> {
        match self.entries.get(entry_idx) {
            Some((id, _)) => Some(text_editor::focus(id.clone())),
            None => None,
        }
    }

    pub fn perform_editor_action(&mut self, action: text_editor::Action, target_id: text_editor::Id) {
        match self.entries.iter_mut().find(|(id, _)| id == &target_id) {
            Some((_, content)) => content.perform(action),
            None => {}
        };
    }

    pub fn display_editors(&self) -> Column<Message> {
        let mut entries = Column::new();
        for (id, content) in &self.entries {
            entries = entries.push(
                text_editor(content)
                    .placeholder("notes")
                    .id(id.clone())
                    .on_action(move |a| Message::EditorActivate(a, id.clone())),
            );
        }
        entries
    }

    pub fn save_and_exit(&self) -> Task<Message> {
        let all_notes: Vec<String> = self
            .entries
            .iter()
            .map(|(_, content)| content.text())
            .collect();
        let text_notes = all_notes.join("\n");
        let state = Savefile {
            log_entries: all_notes,
        };
        if let Some(state_file) = &self.state_file {
            match state.write_to_json(state_file) {
                Err(e) => {
                    println!("{e}");
                    println!("{state:?}");
                },
                _ => (),
            };
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
}
