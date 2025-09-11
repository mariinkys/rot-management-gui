// SPDX-License-Identifier: GPL-3.0-only

use anywho::anywho;

/// Rollback the system using: pkexec rpm-ostree rollback
pub async fn rollback() -> Result<(), anywho::Error> {
    use tokio::process::Command;

    // pkexec first
    let output = Command::new("pkexec")
        .args(["rpm-ostree", "rollback"])
        .output()
        .await?;

    if output.status.success() {
        return Ok(());
    }

    // fallback to distrobox-host-exec
    let output = Command::new("distrobox-host-exec")
        .args(["pkexec", "rpm-ostree", "rollback"])
        .output()
        .await?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        Err(anywho!(
            "distrobox-host-exec failed: {}",
            if !stderr.is_empty() {
                stderr.trim()
            } else {
                stdout.trim()
            }
        ))
    }
}
