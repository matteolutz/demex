use std::fmt;

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub enum DemexGlobalDialogEntryType {
    Error,
    Warn,
    Info,
}

impl DemexGlobalDialogEntryType {
    pub fn title(&self) -> &str {
        match self {
            Self::Error => "Error",
            Self::Warn => "Warning",
            Self::Info => "Info",
        }
    }

    pub fn color(&self) -> egui::Color32 {
        match self {
            Self::Error => egui::Color32::LIGHT_RED,
            Self::Warn => egui::Color32::YELLOW,
            Self::Info => egui::Color32::LIGHT_BLUE,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DemexGlobalDialogEntry {
    entry_type: DemexGlobalDialogEntryType,
    message: String,
    time: chrono::DateTime<chrono::Local>,
    link: Option<String>,
}

impl DemexGlobalDialogEntry {
    pub fn new(
        entry_type: DemexGlobalDialogEntryType,
        message: String,
        link: Option<String>,
    ) -> Self {
        Self {
            entry_type,
            message,
            time: chrono::offset::Local::now(),
            link,
        }
    }

    pub fn color(&self) -> egui::Color32 {
        self.entry_type.color()
    }

    pub fn time(&self) -> chrono::DateTime<chrono::Local> {
        self.time
    }

    pub fn entry_type(&self) -> DemexGlobalDialogEntryType {
        self.entry_type
    }

    pub fn link(&self) -> Option<&String> {
        self.link.as_ref()
    }

    pub fn error(error: &dyn std::error::Error) -> Self {
        Self::new(DemexGlobalDialogEntryType::Error, error.to_string(), None)
    }

    pub fn error_str(error: String) -> Self {
        Self::new(DemexGlobalDialogEntryType::Error, error, None)
    }

    pub fn warn(warn: &str) -> Self {
        Self::new(DemexGlobalDialogEntryType::Warn, warn.to_string(), None)
    }

    pub fn info(info: &str) -> Self {
        Self::new(DemexGlobalDialogEntryType::Info, info.to_string(), None)
    }

    pub fn info_with_link(info: &str, link: &str) -> Self {
        Self::new(
            DemexGlobalDialogEntryType::Info,
            info.to_string(),
            Some(link.to_string()),
        )
    }
}

impl DemexGlobalDialogEntry {
    pub fn window_ui(&self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.label(
                egui::RichText::from(self.to_string())
                    .strong()
                    .size(20.0)
                    .color(self.color()),
            );

            if let Some(link) = self.link() {
                if ui.link(link).clicked() {
                    let _ = open::that(link);
                }
            }
        });
    }
}

impl fmt::Display for DemexGlobalDialogEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {}: {}",
            self.time.format("%H:%M:%S"),
            self.entry_type.title(),
            self.message
        )
    }
}
