use crate::homescreen::{Message, NoteEditors};

use iced::Task;
use iced::keyboard::{Key, Modifiers, key::Named};

#[derive(Debug, PartialEq, Copy, Clone)]
enum Mode {
    Command,
    SelectEdit,
    SelectExit,
    Edit,
    Exit,
}

#[derive(Debug, Copy, Clone)]
pub struct ModeManager {
    active_mode: Mode,
}

struct StateTransition {
    next_mode: Mode,
    transition_task: Task<Message>,
}

impl ModeManager {
    pub fn new() -> ModeManager {
        ModeManager {
            active_mode: Mode::Command,
        }
    }

    fn handle_cmd_keypress(&self, k: Key, screen: &mut NoteEditors) -> StateTransition {
        let c = if let Key::Character(c) = k {
            c.to_string()
        } else {
            return StateTransition {
                next_mode: self.active_mode,
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
            return StateTransition {
                next_mode: Mode::Edit,
                transition_task: screen.add_note(),
            };
        }
        if c == "x" {
            return StateTransition {
                next_mode: Mode::Exit,
                transition_task: screen.save_and_exit(),
            };
        }
        StateTransition {
            next_mode: self.active_mode,
            transition_task: Task::none(),
        }
    }

    fn handle_selectedit_keypress(&self, k: Key, screen: &NoteEditors) -> StateTransition {
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

        match screen.focus_entry(idx) {
            Some(task) => StateTransition {
                next_mode: Mode::Edit,
                transition_task: task,
            },
            None => fallback_to_cmd_mode,
        }
    }

    fn handle_selectexit_keypress(&self, k: Key, screen: &NoteEditors) -> StateTransition {
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

        StateTransition {
            next_mode: Mode::Exit,
            transition_task: screen.save_and_exit(),
        }
    }

    fn handle_edit_keypress(
        &self,
        k: Key,
        modifier: Modifiers,
        screen: &NoteEditors,
    ) -> StateTransition {
        let no_transition = StateTransition {
            next_mode: self.active_mode,
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
            return StateTransition {
                next_mode: Mode::Exit,
                transition_task: screen.save_and_exit(),
            };
        }

        return no_transition;
    }

    pub fn handle_keypress(
        &mut self,
        k: Key,
        m: Modifiers,
        editors: &mut NoteEditors,
    ) -> Task<Message> {
        let transition = match k {
            Key::Named(nk) => {
                let next_mode = if nk == Named::Escape {
                    Mode::Command
                } else {
                    self.active_mode
                };
                StateTransition {
                    next_mode,
                    transition_task: Task::none(),
                }
            }
            Key::Character(_) => match self.active_mode {
                Mode::Command => self.handle_cmd_keypress(k, editors),
                Mode::SelectEdit => self.handle_selectedit_keypress(k, editors),
                Mode::SelectExit => self.handle_selectexit_keypress(k, editors),
                Mode::Edit => self.handle_edit_keypress(k, m, editors),
                Mode::Exit => StateTransition {
                    next_mode: self.active_mode,
                    transition_task: Task::none(),
                },
            },
            Key::Unidentified => StateTransition {
                next_mode: self.active_mode,
                transition_task: Task::none(),
            },
        };

        self.active_mode = transition.next_mode;
        transition.transition_task
    }
}
