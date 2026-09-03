// build.rs
use std::process::Command;

fn main() {
    // Only gate release builds; debug/dev builds shouldn't need a matching tag.
    if std::env::var("PROFILE").as_deref() != Ok("release") {
        return;
    }

    // Published/vendored crates ship without .git — skip rather than hard-fail there.
    if !std::path::Path::new(".git").exists() {
        return;
    }

    let cargo_ver = env!("CARGO_PKG_VERSION");

    // --exact fails (non-zero) if HEAD isn't tagged, which is what we want.
    let out = Command::new("git")
        .args(["describe", "--tags", "--exact-match"])
        .output()
        .expect("git not available");

    if !out.status.success() {
        panic!("release build blocked: HEAD is not tagged");
    }

    let tag = String::from_utf8_lossy(&out.stdout).trim().to_string();
    // Strip a leading `v` so `v1.2.3` matches Cargo.toml's `1.2.3`.
    let tag = tag.strip_prefix('v').unwrap_or(&tag);

    if tag != cargo_ver {
        panic!("version mismatch: tag {tag} != Cargo.toml {cargo_ver}");
    }
}
