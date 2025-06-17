mod homescreen;
mod savestate;

use crate::homescreen::{Homescreen,Message};

fn main() -> iced::Result {
    iced::application("polar tales", Homescreen::update, Homescreen::view)
        // on_key_press might make more sense, but TextEditor widgets eat many
        // of the inputs instead of passing them through, whereas on_key_release
        // is a bit more generous with the events that get passed through. This
        // makes things like switching modes with escape key, shortcuts like
        // ctrl-s, etc a bit more ergonomic for the user, since otherwise they
        // would have to unfocus the text_editor widget before those shortcuts
        // could become available.
        .subscription(|_| iced::keyboard::on_key_release(|k, m| Some(Message::Keypress(k, m))))
        .run()
}
