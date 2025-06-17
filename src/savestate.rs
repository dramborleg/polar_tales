use serde::{Deserialize, Serialize};

use std::error;
use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug)]
pub enum SavestateError {
    Io(io::Error),
    NoParent,
    FailedSerialization(serde_json::Error),
}

impl fmt::Display for SavestateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SavestateError::Io(err) => write!(f, "IO Error: {}", err),
            SavestateError::NoParent => write!(f, "Parent dir does not exist"),
            SavestateError::FailedSerialization(err) => {
                write!(f, "Failed to serialize to json {}", err)
            }
        }
    }
}

impl error::Error for SavestateError {
    fn cause(&self) -> Option<&dyn error::Error> {
        match self {
            SavestateError::Io(err) => Some(err),
            SavestateError::NoParent => None,
            SavestateError::FailedSerialization(err) => Some(err),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Savefile {
    pub log_entries: Vec<String>,
}

impl Savefile {
    pub fn read_from_json(p: impl AsRef<Path>) -> Option<Savefile> {
        let Ok(text) = fs::read_to_string(p) else {
            return None;
        };

        match serde_json::from_str::<Savefile>(&text) {
            Ok(state) => Some(state),
            Err(_) => None,
        }
    }

    pub fn write_to_json(&self, p: impl AsRef<Path>) -> Result<(), SavestateError> {
        let json_notes =
            serde_json::to_string_pretty(&self).map_err(SavestateError::FailedSerialization)?;

        let Some(parent) = p.as_ref().parent() else {
            return Err(SavestateError::NoParent);
        };
        fs::create_dir_all(parent).map_err(SavestateError::Io)?;
        let newline_terminated_contents = format!("{json_notes}\n");
        fs::write(p, newline_terminated_contents).map_err(SavestateError::Io)?;
        Ok(())
    }
}
