use std::{path::PathBuf, sync::OnceLock};

static HOME_DIR: OnceLock<PathBuf> = OnceLock::new();
static CONFIG_DIR: OnceLock<PathBuf> = OnceLock::new();
static DATA_DIR: OnceLock<PathBuf> = OnceLock::new();

fn home_dir() -> &'static PathBuf {
    HOME_DIR.get_or_init(|| dirs::home_dir().expect("Failed to get home dir"))
}

fn config_dir() -> &'static PathBuf {
    CONFIG_DIR.get_or_init(|| {
        if cfg!(target_os = "windows") {
            dirs::config_dir()
                .expect("Failed to get RoamingAppData")
                .join("demex")
        } else if cfg!(any(target_os = "linux", target_os = "freebsd")) {
            if let Ok(flatpak_xdg_config) = std::env::var("FLATPAK_XDG_CONFIG_HOME") {
                flatpak_xdg_config.into()
            } else {
                dirs::config_dir().expect("failed to determine XDG_CONFIG_HOME directory")
            }
            .join("demex")
        } else {
            home_dir().join(".config").join("demex")
        }
    })
}

fn data_dir() -> &'static PathBuf {
    DATA_DIR.get_or_init(|| {
        if cfg!(target_os = "macos") {
            home_dir().join("Library/Application Support/demex")
        } else if cfg!(any(target_os = "linux", target_os = "freebsd")) {
            if let Ok(flatpak_xdg_data) = std::env::var("FLATPAK_XDG_DATA_HOME") {
                flatpak_xdg_data.into()
            } else {
                dirs::data_local_dir().expect("failed to determine XDG_DATA_HOME directory")
            }
            .join("demex")
        } else if cfg!(target_os = "windows") {
            dirs::data_local_dir()
                .expect("failed to determine LocalAppData directory")
                .join("demex")
        } else {
            config_dir().clone() // Fallback
        }
    })
}

pub fn fixture_types() -> &'static PathBuf {
    static FIXTURE_TYPES: OnceLock<PathBuf> = OnceLock::new();
    FIXTURE_TYPES.get_or_init(|| data_dir().join("fixture_types"))
}

pub fn themes_dir() -> &'static PathBuf {
    static THEMES_DIR: OnceLock<PathBuf> = OnceLock::new();
    THEMES_DIR.get_or_init(|| config_dir().join("themes"))
}

pub fn read_or_create_dir(dir: &PathBuf) -> std::io::Result<std::fs::ReadDir> {
    let res = std::fs::read_dir(dir);
    let error = match res {
        Ok(res) => return Ok(res),
        Err(err) => err,
    };

    if !matches!(error.kind(), std::io::ErrorKind::NotFound) {
        return Err(error);
    }

    log::debug!("Directory {} doesn't exist, creating it...", dir.display());
    std::fs::create_dir_all(dir)?;

    std::fs::read_dir(dir)
}
