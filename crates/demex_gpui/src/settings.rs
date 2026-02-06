use std::fs::File;

use dirs::config_dir;
use gpui::{Bounds, DisplayId, Global, Pixels, WindowBounds};
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum DemexWindowBounds {
    Windowed(Bounds<Pixels>),
    Maximized(Bounds<Pixels>),
    Fullscreen(Bounds<Pixels>),
}

impl From<WindowBounds> for DemexWindowBounds {
    fn from(value: WindowBounds) -> Self {
        match value {
            WindowBounds::Windowed(bounds) => Self::Windowed(bounds),
            WindowBounds::Maximized(bounds) => Self::Maximized(bounds),
            WindowBounds::Fullscreen(bounds) => Self::Fullscreen(bounds),
        }
    }
}

impl From<DemexWindowBounds> for WindowBounds {
    fn from(value: DemexWindowBounds) -> Self {
        match value {
            DemexWindowBounds::Windowed(bounds) => Self::Windowed(bounds),
            DemexWindowBounds::Maximized(bounds) => Self::Maximized(bounds),
            DemexWindowBounds::Fullscreen(bounds) => Self::Fullscreen(bounds),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct DemexWindowSettings {
    pub bounds: Option<DemexWindowBounds>,
    pub display_id: Option<u32>,
}

const DEMEX_SETTINGS_FILE: &str = "settings.json";

#[derive(Default, Serialize, Deserialize)]
pub struct DemexSettings {
    window_settings: Vec<DemexWindowSettings>,
}

impl DemexSettings {
    pub fn load() -> DemexSettings {
        let settings_path = config_dir().unwrap().join(DEMEX_SETTINGS_FILE);
        log::debug!("Loading settings from {}", settings_path.display());

        if settings_path.is_file() {
            let file = File::open(settings_path).unwrap();
            return serde_json::from_reader(file).unwrap_or_default();
        }

        log::debug!("No settings file found, creating default settings");
        let settings = Self::default();

        let file = File::create(settings_path).unwrap();

        #[cfg(debug_assertions)]
        serde_json::to_writer_pretty(file, &settings).unwrap();
        #[cfg(not(debug_assertions))]
        serde_json::to_writer(file, &settings).unwrap();

        settings
    }

    pub fn save(&self) {
        let settings_path = config_dir().unwrap().join(DEMEX_SETTINGS_FILE);
        log::debug!("Saving settings to {}", settings_path.display());

        let file = File::create(settings_path).unwrap();

        #[cfg(debug_assertions)]
        serde_json::to_writer_pretty(file, &self).unwrap();
        #[cfg(not(debug_assertions))]
        serde_json::to_writer(file, &self).unwrap();
    }

    fn ensure_window_settings(&mut self, idx: usize) {
        if idx >= self.window_settings.len() {
            self.window_settings
                .resize(idx + 1, DemexWindowSettings::default());
        }
    }

    pub fn update_window_bounds(&mut self, idx: usize, bounds: impl Into<DemexWindowBounds>) {
        self.ensure_window_settings(idx);
        self.window_settings[idx].bounds = Some(bounds.into());
    }

    pub fn update_window_display_id(&mut self, idx: usize, display_id: DisplayId) {
        self.ensure_window_settings(idx);
        self.window_settings[idx].display_id = Some(display_id.into());
    }

    pub fn window_settings(&self, idx: usize) -> Option<DemexWindowSettings> {
        self.window_settings.get(idx).cloned()
    }
}

impl Global for DemexSettings {}
