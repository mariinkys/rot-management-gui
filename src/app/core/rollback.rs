// SPDX-License-Identifier: GPL-3.0-only

use anywho::anywho;
use std::os::unix::process::ExitStatusExt;
use std::process::ExitStatus;

/// Rollback the system using: pkexec rpm-ostree rollback
pub async fn rollback() -> Result<(), anywho::Error> {
    use tokio::process::Command;

    let output = if super::is_running_in_distrobox() {
        Command::new("distrobox-host-exec")
            .args(["pkexec", "rpm-ostree", "rollback"])
            .output()
            .await
    } else {
        Command::new("pkexec")
            .args(["rpm-ostree", "rollback"])
            .output()
            .await
    };

    match output {
        Ok(output) => {
            if output.status.success() {
                Ok(())
            } else {
                if output.status == ExitStatus::from_raw(32256) {
                    return Err(anywho!("Permision denied"));
                }

                let stderr = String::from_utf8_lossy(&output.stderr);
                let stdout = String::from_utf8_lossy(&output.stdout);

                Err(anywho!(
                    "{}",
                    if !stderr.is_empty() {
                        stderr.trim()
                    } else {
                        stdout.trim()
                    }
                ))
            }
        }
        Err(err) => Err(anywho!("{}", err)),
    }
}
