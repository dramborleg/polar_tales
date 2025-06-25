# polar tales

A minimally invasive app for taking notes throughout the day, written in [rust](https://www.rust-lang.org/) and [iced](https://iced.rs/).

## Motivation

At the end of each day, I like to write down a short overview of the things I worked on. Sometimes I make notes of different approaches I tried or other bits of information if they seem important, but at the end of the day it is harder to remember than if I did it while I was working on it. The goal of this app is to have a reminder to write down little notes throughout the day without being too disruptive to the workflow. It also gives rough estimates of how much time was spent on each task.

## Usage

polar tales is intended to be run at a periodic intervals so that when it starts up, it serves as a reminder to update the notes. This periodic launch is not implemented within polar tales (yet?) so a small shell script could be useful

```sh
#!/bin/bash
rm ~/.local/state/polartales/state.json # clear yesterday's notes
while true; do
  sleep 1200
  polartales
done
```

There is a command mode and an edit mode, and the application is primarily keyboard driven. `Esc` can be pressed at any time to return to command mode, and in edit mode, `ctrl-s` saves and exits. Note that while using the mouse to select a notes entry will allow it to be edited, selecting in this manner does *not* (yet) put the application into edit mode. There are a few actions in command mode:

- `n` - create a new note entry
- `e<0-9>` - edit an existing note, eg `e0` to edit the first note
- `x` - immediately exit

polar tales always attempts to save the notes into a state file before exiting to use as the starting state in future runs (saves have not yet been tested at all on windows/mac). It unconditionally copies the contents to the clipboard before exiting, so be careful if this is undesirable (it may eventually be configurable).

The time spent on each task is also estimated by polar tales. The duration since the previous exit and current exit is added to the task which was last in focus before the application exits, and these time estimates continue accumulating until the state file is cleared. This may influence how large of a sleep interval is chosen between runs, since a larger sleep interval will result in less granular time estimates as the entire interval is attributed to only one task.
