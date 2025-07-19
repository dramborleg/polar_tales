use crate::homescreen::Message;
use crate::savestate::{Savefile, SavefileLogEntry};

use dirs::state_dir;

use iced::Task;
use iced::clipboard;
use iced::widget::{Column, text_editor};
use iced::window;

use indexmap::IndexMap;

use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const SEC_PER_MIN: u64 = 60;

#[derive(Debug)]
pub struct NoteEditors {
    state_file: Option<PathBuf>,
    entries: IndexMap<text_editor::Id, (text_editor::Content, Duration)>,
    last_focused_id: Option<text_editor::Id>,
    last_app_exit: SystemTime,
}

impl NoteEditors {
    pub fn new() -> NoteEditors {
        let state_file = state_dir().map(|sd| sd.join("polartales/state.json"));
        let empty_savestate = Savefile {
            log_entries: Vec::new(),
            unix_time_last_exit: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or(Duration::ZERO)
                .as_secs(),
            last_focused_idx: None,
        };
        let savestate = if let Some(state_file) = &state_file {
            Savefile::read_from_json(state_file).unwrap_or(empty_savestate)
        } else {
            empty_savestate
        };

        let entries: IndexMap<text_editor::Id, (text_editor::Content, Duration)> = savestate
            .log_entries
            .iter()
            .map(|entry| {
                let mut content = text_editor::Content::with_text(&entry.notes);
                content.perform(text_editor::Action::Move(text_editor::Motion::DocumentEnd));
                let time_spent = Duration::from_secs(entry.minutes_spent * SEC_PER_MIN);
                (content, time_spent)
            })
            .map(|entry| (text_editor::Id::unique(), entry))
            .collect();

        let last_app_exit = UNIX_EPOCH + Duration::from_secs(savestate.unix_time_last_exit);

        let last_focused_id: Option<text_editor::Id> = if let Some(idx) = savestate.last_focused_idx
        {
            entries.get_index(idx as usize).map(|(k, _)| k.clone())
        } else {
            None
        };

        NoteEditors {
            state_file,
            entries,
            last_focused_id,
            last_app_exit,
        }
    }

    fn set_focus(&mut self, id: text_editor::Id) -> Task<Message> {
        self.last_focused_id = Some(id.clone());
        text_editor::focus(id)
    }

    pub fn add_note(&mut self) -> Task<Message> {
        let id = text_editor::Id::unique();
        self.entries
            .insert(id.clone(), (text_editor::Content::new(), Duration::ZERO));
        self.set_focus(id)
    }

    pub fn focus_entry(&mut self, entry_idx: usize) -> Option<Task<Message>> {
        if let Some((id, _)) = self.entries.get_index(entry_idx) {
            return Some(self.set_focus(id.clone()));
        }
        None
    }

    pub fn focus_mru_entry(&mut self) -> Option<Task<Message>> {
        if let Some(id) = &self.last_focused_id {
            return Some(self.set_focus(id.clone()));
        }
        None
    }

    pub fn perform_editor_action(
        &mut self,
        action: text_editor::Action,
        target_id: text_editor::Id,
    ) {
        if let Some((content, _)) = self.entries.get_mut(&target_id) {
            content.perform(action)
        }
    }

    pub fn display_editors(&self) -> Column<Message> {
        let mut editors = Column::new();
        for (id, (content, _)) in &self.entries {
            editors = editors.push(
                text_editor(content)
                    .placeholder("notes")
                    .id(id.clone())
                    .on_action(move |a| Message::EditorActivate(a, id.clone())),
            );
        }
        editors
    }

    pub fn save_and_exit(&self, write_clipboard: bool) -> Task<Message> {
        let cur_task_interval = SystemTime::now()
            .duration_since(self.last_app_exit)
            .unwrap_or(Duration::ZERO);

        let log_entries: Vec<SavefileLogEntry> = self
            .entries
            .iter()
            .map(|(id, (content, time_spent))| {
                let additional_time_spent = if let Some(last_focused_id) = &self.last_focused_id {
                    if id == last_focused_id {
                        cur_task_interval
                    } else {
                        Duration::ZERO
                    }
                } else {
                    Duration::ZERO
                };
                let minutes_spent = (*time_spent + additional_time_spent).as_secs() / SEC_PER_MIN;
                SavefileLogEntry {
                    notes: content.text(),
                    minutes_spent,
                }
            })
            .collect();
        let unix_time_last_exit = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs();
        let last_focused_idx = if let Some(id) = &self.last_focused_id {
            self.entries.get_index_of(id).map(|idx| idx as u64)
        } else {
            None
        };
        let state = Savefile {
            log_entries,
            unix_time_last_exit,
            last_focused_idx,
        };
        if let Some(state_file) = &self.state_file {
            if let Err(e) = state.write_to_json(state_file) {
                println!("{e}");
                println!("{state:?}");
            };
        } else {
            println!("{state:?}");
        };

        // iced::exit() is tempting to return without closing the window first,
        // but this results in a segfault in iced on some platforms, so closing
        // the window first is cleaner, at least for now.
        // https://github.com/iced-rs/iced/issues/2983
        let exit_task = window::get_latest()
            .and_then(window::close)
            .chain(iced::exit());

        if !write_clipboard {
            return exit_task;
        }

        let clipboard_notes = state
            .log_entries
            .iter()
            .map(|entry| entry.minutes_spent.to_string() + "m: " + &entry.notes)
            .collect::<Vec<String>>()
            .join("\n\n");
        clipboard::write(clipboard_notes).chain(exit_task)
    }
}
