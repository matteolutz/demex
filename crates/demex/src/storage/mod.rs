use std::path::PathBuf;

#[allow(unused_variables)]
fn storage_dir(app_id: &str, path: &str) -> PathBuf {
    let dir;

    dir = std::env::current_dir().unwrap().join("demex").join(path);

    if !dir.exists() {
        std::fs::create_dir_all(dir.clone()).unwrap();
    }

    dir
}

pub fn fixture_types(app_id: &str) -> PathBuf {
    storage_dir(app_id, "fixtures")
}
