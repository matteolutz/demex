use dialog::DemexGlobalDialogEntry;

use crate::{
    lexer::token::Token,
    parser::nodes::action::{result::ActionRunResult, Action},
};

pub mod dialog;

pub enum DemexLogEntryType {
    Info(String),
    DialogEntry(DemexGlobalDialogEntry),
    CommandEntry(Vec<Token>),
    CommandFailedEntry(String),
    ActionEntrySuccess(Action, ActionRunResult),
    ActionEntryFailed(Action, String),
    Error(String),
}

impl std::fmt::Display for DemexLogEntryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DemexLogEntryType::Info(info) => write!(f, "[INFO]: {}", info),
            DemexLogEntryType::DialogEntry(entry) => write!(f, "[DLG]: {}", entry),
            DemexLogEntryType::CommandEntry(tokens) => {
                write!(f, "[CMD]: ")?;

                // omit the Eof token
                for token in tokens.iter().take(tokens.len() - 1) {
                    write!(f, "{} ", token)?;
                }
                Ok(())
            }
            DemexLogEntryType::CommandFailedEntry(err) => write!(f, "[CMD][FAIL]: {}", err),
            DemexLogEntryType::ActionEntrySuccess(action, res) => {
                write!(f, "[ACT][SUCC]: {:?}\n\t> {:?}", action, res)
            }
            DemexLogEntryType::ActionEntryFailed(action, err) => {
                write!(f, "[ACT][FAIL]: {:?}\n\t> {}", action, err)
            }
            DemexLogEntryType::Error(err) => {
                write!(f, "[DEMEX][ERR]: {err}")
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

    pub fn color(&self) -> ecolor::Color32 {
        match &self.entry_type {
            DemexLogEntryType::Info(_) => ecolor::Color32::DARK_GREEN,
            DemexLogEntryType::DialogEntry(_) => ecolor::Color32::DARK_GRAY,
            DemexLogEntryType::CommandEntry(_) => ecolor::Color32::LIGHT_BLUE,
            DemexLogEntryType::CommandFailedEntry(_) => ecolor::Color32::LIGHT_RED,
            DemexLogEntryType::ActionEntrySuccess(_, _) => ecolor::Color32::LIGHT_GREEN,
            DemexLogEntryType::ActionEntryFailed(_, _) => ecolor::Color32::LIGHT_RED,
            DemexLogEntryType::Error(_) => ecolor::Color32::RED,
        }
    }
}

impl std::fmt::Display for DemexLogEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.time.format("%H:%M:%S"), self.entry_type)
    }
}
