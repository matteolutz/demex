pub const VERSION_STR: &str = env!("CARGO_PKG_VERSION");

pub fn demex_version() -> (u8, u8, u8) {
    let version = VERSION_STR.split('.').collect::<Vec<&str>>();
    let major = version[0].parse::<u8>().unwrap();
    let minor = version[1].parse::<u8>().unwrap();
    let patch = version[2].parse::<u8>().unwrap();
    (major, minor, patch)
}
