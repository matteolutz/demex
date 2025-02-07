use std::fmt;

use crate::{
    lexer::token::Token,
    parser::nodes::action::{result::ActionRunResult, Action},
};

use super::DemexGlobalDialogEntry;

pub enum DemexLogEntryType {
    DialogEntry(DemexGlobalDialogEntry),
    CommandEntry(Vec<Token>),
    CommandFailedEntry(String),
    ActionEntrySuccess(Action, ActionRunResult),
    ActionEntryFailed(Action, String),
}

impl fmt::Display for DemexLogEntryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DemexLogEntryType::DialogEntry(entry) => write!(f, "[DLG]: {}", entry),
            DemexLogEntryType::CommandEntry(tokens) => {
                write!(f, "[CMD]: ")?;
                for i in 0..tokens.len() - 1 {
                    write!(f, "{} ", tokens[i])?;
                }
                Ok(())
            }
            DemexLogEntryType::CommandFailedEntry(err) => write!(f, "[CMD][FAIL]: {}", err),
            DemexLogEntryType::ActionEntrySuccess(action, res) => {
                write!(f, "[ACT][SUCC]: {:?} -> {:?}", action, res)
            }
            DemexLogEntryType::ActionEntryFailed(action, err) => {
                write!(f, "[ACT][FAIL]: {:?} -> {}", action, err)
            }
        }
    }
}

pub struct DemexLogEntry {
    entry_type: DemexLogEntryType,
    time: chrono::DateTime<chrono::Local>,
}

impl DemexLogEntry {
    pub fn new(entry_type: DemexLogEntryType) -> Self {
        Self {
            entry_type,
            time: chrono::Local::now(),
        }
    }

    pub fn time(&self) -> &chrono::DateTime<chrono::Local> {
        &self.time
    }

    pub fn color(&self) -> egui::Color32 {
        match &self.entry_type {
            DemexLogEntryType::DialogEntry(_) => egui::Color32::DARK_GRAY,
            DemexLogEntryType::CommandEntry(_) => egui::Color32::LIGHT_BLUE,
            DemexLogEntryType::CommandFailedEntry(_) => egui::Color32::LIGHT_RED,
            DemexLogEntryType::ActionEntrySuccess(_, _) => egui::Color32::LIGHT_GREEN,
            DemexLogEntryType::ActionEntryFailed(_, _) => egui::Color32::LIGHT_RED,
        }
    }
}

impl fmt::Display for DemexLogEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.time.format("%H:%M:%S"), self.entry_type)
    }
}
