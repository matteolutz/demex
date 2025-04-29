use std::path::PathBuf;

fn storage_dir(app_id: &str, path: &str) -> PathBuf {
    let dir = eframe::storage_dir(app_id).unwrap().join(path);

    if !dir.exists() {
        std::fs::create_dir_all(dir.clone()).unwrap();
    }

    dir
}

pub fn fixture_types(app_id: &str) -> PathBuf {
    storage_dir(app_id, "fixtures")
}
