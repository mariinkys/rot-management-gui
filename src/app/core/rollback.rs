// SPDX-License-Identifier: GPL-3.0-only

use anywho::anywho;
use std::os::unix::process::ExitStatusExt;
use std::process::ExitStatus;

use crate::app::core::run_command;

/// Rollback the system using: pkexec rpm-ostree rollback
pub async fn rollback() -> Result<(), anywho::Error> {
    let output = run_command("pkexec", &["rpm-ostree", "rollback"]).await;

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
