# polar tales

A minimally invasive app for taking notes throughout the day, written in [rust](https://www.rust-lang.org/) and [iced](https://iced.rs/).

## Motivation

At the end of each day, I like to write down a short overview of the things I worked on. Sometimes I make notes of different approaches I tried or other bits of information if they seem important, but at the end of the day it is harder to remember than if I did it while I was working on it. The goal of this app is to have a reminder to write down little notes throughout the day without being too disruptive to the workflow. It also gives rough estimates of how much time was spent on each task.

## Usage

polar tales is intended to be run at a periodic intervals so that by stealing focus, the user is reminded to update the notes. The time period between application runs is tracked and used to estimate the time spent working on each task. To accomplish this, the duration between the previous and current application launch is added to the last task which is focused/edited before exiting.

The periodic interval launch is not implemented within polar tales (yet?) so this is an example script which could be used to kick it off. Increasing the sleep time results in fewer interruptions, but may give less accurate time estimates since the entire interval is attributed to just one task.

```sh
#!/bin/bash

# clear yesterday's notes
rm ~/.local/state/polartales/state.json # or $XDG_STATE_HOME if it is defined
while true; do
  sleep 1200
  polartales
done
```

The application is primarily keyboard driven and has a command mode (the default mode) and an edit mode. Note that while using the mouse to select a note entry will allow it to be edited, selecting in this manner does *not* (yet) put the application into edit mode so it may exhibit erratic behavior.

Also note that polar tales implements a one-second start delay - after starting, no key presses are handled until one second of time has passed. This helps prevent inputting unintended commands in case one was in the middle of typing when the keyboard focus was stolen.

`Esc` can be pressed at any time to return to command mode.

command mode commands:

- `n` - create a new note entry
- `e<0-9>` - edit an existing note, eg `e0` to edit the first note
- `s<0-9>` - select an existing note to add the time estimate duration to, then save and exit
- `x` - save and exit; time estimate is added to the last edited task; if no task was edited in this polar tales instance, it is added to the last edited task from the previous instance of polar tales
- `ctrl-c` - copy all note entries to the clipboard, then save and exit; time estimate is added to the last edited task

edit mode commands:

- `ctrl-s` - save and exit
- `ctrl-c` - copy all the note entries to the clipboard, then save and exit

## Screenshots

Eventually it would be nice to add some visual differentiation for each mode and perhaps make the entry selection easier as well, for now it is just a basic interface:

![Screenshot of the main screen in command mode.](/screenshots/command_mode.png)
