// crates/tachikoma-common-core/build.rs
fn main() {
    // Re-run if build script changes
    println!("cargo:rerun-if-changed=build.rs");

    // Embed git info
    if let Ok(output) = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
    {
        if output.status.success() {
            let git_hash = String::from_utf8_lossy(&output.stdout);
            println!("cargo:rustc-env=GIT_HASH={}", git_hash.trim());
        }
    }

    // Build timestamp
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ");
    println!("cargo:rustc-env=BUILD_TIME={}", now);
}
