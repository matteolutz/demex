use std::process::Command;

fn get_short_commit_hash() -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["rev-parse", "--short", "HEAD"])
        .output()?;
    let commit_hash = String::from_utf8(output.stdout)?;
    Ok(commit_hash)
}

fn main() {
    println!(
        "cargo:rustc-env=GIT_HASH={}",
        get_short_commit_hash().unwrap_or_else(|_| "-".to_string())
    );
}
