# polar tales

A minimally invasive app for taking notes throughout the day, written in [rust](https://www.rust-lang.org/) and [iced](https://iced.rs/).

## Motivation

At the end of each day, I like to write down a short overview of the things I worked on. Sometimes I make notes of different approaches I tried or other bits of information if they seem important, but at the end of the day it is harder to remember than if I did it while I was working on it. The goal of this app is to have a reminder to write down little notes throughout the day without being too disruptive to the workflow. It also gives rough estimates of how much time was spent on each task.

## Usage

polar tales is intended to be run at a periodic intervals so that when it steals focus, the user is reminded to update the notes. The time period between application runs is also tracked and used to create estimates of the time spent working on each task. To accomplish this, the duration between the previous and current launch is added to the last task which is focused/edited before exiting. The sleep interval can be adjusted up or down based on the desired accuracy of time estimates (the entire interval is attributed to just one task) weighed against how frequently one is willing to tolerate interruptions to their workflow.

The periodic interval launch is not implemented within polar tales (yet?) so this is an example script which could be used to kick it off

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

`Esc` can be pressed at any time to return to command mode.

command mode commands:

- `n` - create a new note entry
- `e<0-9>` - edit an existing note, eg `e0` to edit the first note
- `s<0-9>` - select an existing note to add the time estimate duration to, then save and exit
- `x` - save and exit

edit mode commands:

- `ctrl-s` - save and exit
- `ctrl-c` - copy all the note entries to the clipboard, then save and exit
