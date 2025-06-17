use dirs::state_dir;

use iced::Element;
use iced::Task;
use iced::clipboard;
use iced::keyboard::key::Named;
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
enum Message {
    Keypress(Key, Modifiers),
    EditorActivate(text_editor::Action, text_editor::Id),
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum Mode {
    Command,
    SelectEdit,
    SelectExit,
    Edit,
    Exit,
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
struct Counter {
    state_file: Option<PathBuf>,
    edit_entries: Vec<(text_editor::Id, text_editor::Content)>,
    mode: Mode,
}

struct StateTransition {
    next_mode: Mode,
    transition_task: Task<Message>,
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
            mode: Mode::Command,
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

    fn save_and_exit(&self) -> StateTransition {
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
        // but this results in a segfault in iced so closing the window first
        // is cleaner, at least for now.
        // https://github.com/iced-rs/iced/issues/2983
        let exit_task = clipboard::write(text_notes).chain(
            window::get_latest()
                .and_then(window::close)
                .chain(iced::exit()),
        );
        StateTransition {
            next_mode: Mode::Exit,
            transition_task: exit_task,
        }
    }

    fn handle_cmd_keypress(&mut self, k: Key) -> StateTransition {
        let c = if let Key::Character(c) = k {
            c.to_string()
        } else {
            return StateTransition {
                next_mode: self.mode,
                transition_task: Task::none(),
            };
        };
        // s to select and exit
        // e to select and edit
        // n to create new entry
        // ctrl + s to save/exit?
        if c == "s" {
            return StateTransition {
                next_mode: Mode::SelectExit,
                transition_task: Task::none(),
            };
        }
        if c == "e" {
            return StateTransition {
                next_mode: Mode::SelectEdit,
                transition_task: Task::none(),
            };
        }
        if c == "n" {
            let id = text_editor::Id::unique();
            self.edit_entries
                .push((id.clone(), text_editor::Content::new()));
            return StateTransition {
                next_mode: Mode::Edit,
                transition_task: text_editor::focus(id),
            };
        }
        if c == "x" {
            return self.save_and_exit();
        }
        StateTransition {
            next_mode: self.mode,
            transition_task: Task::none(),
        }
    }

    fn handle_selectedit_keypress(&self, k: Key) -> StateTransition {
        let fallback_to_cmd_mode = StateTransition {
            next_mode: Mode::Command,
            transition_task: Task::none(),
        };
        let Key::Character(c) = k else {
            return fallback_to_cmd_mode;
        };

        let Ok(idx) = c.parse::<usize>() else {
            return fallback_to_cmd_mode;
        };

        match self.edit_entries.get(idx) {
            Some((id, _)) => StateTransition {
                next_mode: Mode::Edit,
                transition_task: text_editor::focus(id.clone()),
            },
            None => fallback_to_cmd_mode,
        }
    }

    fn handle_selectexit_keypress(&self, k: Key) -> StateTransition {
        let fallback_to_cmd_mode = StateTransition {
            next_mode: Mode::Command,
            transition_task: Task::none(),
        };
        let Key::Character(c) = k else {
            return fallback_to_cmd_mode;
        };

        let Ok(idx) = c.parse::<usize>() else {
            return fallback_to_cmd_mode;
        };

        self.save_and_exit()
    }

    fn handle_edit_keypress(&self, k: Key, modifier: Modifiers) -> StateTransition {
        let no_transition = StateTransition {
            next_mode: self.mode,
            transition_task: Task::none(),
        };
        if !modifier.control() {
            return no_transition;
        }

        let c = if let Key::Character(c) = k {
            c.to_string()
        } else {
            return no_transition;
        };

        if c == "s" {
            return self.save_and_exit();
        }

        return no_transition;
    }

    fn handle_keypress(&mut self, k: Key, m: Modifiers) -> StateTransition {
        match k {
            Key::Named(nk) => {
                let next_mode = if nk == Named::Escape {
                    Mode::Command
                } else {
                    self.mode
                };
                StateTransition {
                    next_mode,
                    transition_task: Task::none(),
                }
            }
            Key::Character(_) => match self.mode {
                Mode::Command => self.handle_cmd_keypress(k),
                Mode::SelectEdit => self.handle_selectedit_keypress(k),
                Mode::SelectExit => self.handle_selectexit_keypress(k),
                Mode::Edit => self.handle_edit_keypress(k, m),
                Mode::Exit => StateTransition {
                    next_mode: self.mode,
                    transition_task: Task::none(),
                },
            },
            Key::Unidentified => StateTransition {
                next_mode: self.mode,
                transition_task: Task::none(),
            },
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        let transition = match message {
            Message::Keypress(k, m) => self.handle_keypress(k, m),
            Message::EditorActivate(action, target_id) => {
                match self
                    .edit_entries
                    .iter_mut()
                    .find(|(id, _)| id == &target_id)
                {
                    Some((_, content)) => content.perform(action),
                    None => {}
                }
                StateTransition {
                    next_mode: self.mode,
                    transition_task: Task::none(),
                }
            }
        };

        self.mode = transition.next_mode;
        transition.transition_task
    }

    fn view(&self) -> Element<Message> {
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

fn main() -> iced::Result {
    iced::application("polar tales", Counter::update, Counter::view)
        // on_key_press might make more sense, but TextEditor widgets eat many
        // of the inputs instead of passing them through, whereas on_key_release
        // is a bit more generous with the events that get passed through. This
        // makes things like switching modes with escape key, shortcuts like
        // ctrl-s, etc a bit more ergonomic for the user.
        .subscription(|_| iced::keyboard::on_key_release(|k, m| Some(Message::Keypress(k, m))))
        .run()
}
