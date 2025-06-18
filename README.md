# polar tales

A minimally invasive app for taking notes throughout the day, written in [rust](https://www.rust-lang.org/) and [iced](https://iced.rs/).

## Motivation

At the end of each day, I like to write down a short overview of the things I worked on. Sometimes I make notes of different approaches I tried or other bits of information if they seem important, but at the end of the day it is harder to remember than if I did it while I was working on it. The goal of this app is to have a reminder to write down little notes throughout the day without being too disruptive to the actual work.

## Usage

Currently, reminders are not built in to this app, so to get that behaviour a small bash loop could be useful

```sh
#!/bin/bash
rm ~/.local/state/polartales/state.json # clear yesterday's notes
while true; do
  sleep 1200
  polartales
done
```

The application itself is keyboard driven with two modes: command and edit. `Esc` can be pressed at any time to return to command mode, and in edit mode, `ctrl-s` saves and exits. There are a few actions in command mode:

- `n` - create a new note entry
- `e<0-9>` - edit an existing note, eg `e0` to edit the first note
- `x` - immediately exit

polar tales always attempts to save the notes before exiting and uses them as the starting state the next time it is run. It also copies the contents to the clipboard before exiting, so be careful if you don't want the clipboard overwritten (may eventually be configurable, or at least back up the clipboard before overwriting it).
