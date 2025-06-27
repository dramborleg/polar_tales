mod mode_manager;
mod note_editors;

use crate::homescreen::mode_manager::ModeManager;
use crate::homescreen::note_editors::NoteEditors;

use iced::Element;
use iced::Task;
use iced::keyboard::{Key, Modifiers};
use iced::widget::text_editor;

use std::time::{Duration, SystemTime};

const INIT_DELAY_TIME: Duration = Duration::from_secs(1);

#[derive(Debug, Clone)]
pub enum Message {
    Keypress(Key, Modifiers),
    EditorActivate(text_editor::Action, text_editor::Id),
}

#[derive(Debug)]
pub struct Homescreen {
    editors: NoteEditors,
    mode_mgr: ModeManager,
    time_at_init: SystemTime,
}

impl Default for Homescreen {
    fn default() -> Self {
        Homescreen {
            mode_mgr: ModeManager::new(),
            editors: NoteEditors::new(),
            time_at_init: SystemTime::now(),
        }
    }
}

impl Homescreen {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        let time_since_init = SystemTime::now()
            .duration_since(self.time_at_init)
            .unwrap_or(INIT_DELAY_TIME);
        if time_since_init < INIT_DELAY_TIME {
            return Task::none();
        }

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
