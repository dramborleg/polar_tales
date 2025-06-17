mod mode_manager;
mod note_editors;

use crate::homescreen::mode_manager::ModeManager;
use crate::homescreen::note_editors::NoteEditors;

use iced::Element;
use iced::Task;
use iced::keyboard::{Key, Modifiers};
use iced::widget::text_editor;

#[derive(Debug, Clone)]
pub enum Message {
    Keypress(Key, Modifiers),
    EditorActivate(text_editor::Action, text_editor::Id),
}

#[derive(Debug)]
pub struct Counter {
    editors: NoteEditors,
    mode_mgr: ModeManager,
}

impl Default for Counter {
    fn default() -> Self {
        Counter {
            mode_mgr: ModeManager::new(),
            editors: NoteEditors::new(),
        }
    }
}

impl Counter {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Keypress(k, m) => self.mode_mgr.handle_keypress(k, m, &mut self.editors),
            Message::EditorActivate(action, target_id) => {
                self.editors.perform_editor_action(action, target_id);
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        self.editors.display_editors().into()
    }
}
