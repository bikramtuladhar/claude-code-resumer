use std::process::Command;

fn main() {
    // Tell Cargo to rerun this if git HEAD changes
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/tags");

    // Try to get version from git tag, fall back to Cargo.toml version
    let version = get_git_tag_version()
        .unwrap_or_else(|| std::env::var("CARGO_PKG_VERSION").unwrap());

    println!("cargo:rustc-env=CS_VERSION={}", version);
}

fn get_git_tag_version() -> Option<String> {
    // Check if we're in a git repo
    let in_git = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !in_git {
        return None;
    }

    // Try to get the exact tag on HEAD
    let output = Command::new("git")
        .args(["describe", "--tags", "--exact-match", "HEAD"])
        .output()
        .ok()?;

    if output.status.success() {
        let tag = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let version = tag.strip_prefix('v').unwrap_or(&tag);
        return Some(version.to_string());
    }

    // Try to get nearest tag with distance (for dev builds between releases)
    let output = Command::new("git")
        .args(["describe", "--tags", "--abbrev=7"])
        .output()
        .ok()?;

    if output.status.success() {
        let desc = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let version = desc.strip_prefix('v').unwrap_or(&desc);

        // Check if dirty
        let dirty = Command::new("git")
            .args(["status", "--porcelain"])
            .output()
            .map(|o| !o.stdout.is_empty())
            .unwrap_or(false);

        if dirty {
            return Some(format!("{}-dirty", version));
        }
        return Some(version.to_string());
    }

    // No tags exist - fall back to None (will use Cargo.toml)
    None
}
