use anywho::anywho;
use std::os::unix::process::ExitStatusExt;
use std::process::ExitStatus;

#[derive(Debug, Clone)]
pub struct SystemUpdate {
    pub version: String,
    pub commit: String,
    pub gpg_signature: String,
    pub sec_advisories: Option<String>,
    pub diff: String,
}

impl SystemUpdate {
    // Check if a [`SystemUpdate`] is available using: rpm-ostree upgrade --check
    pub async fn check() -> Option<SystemUpdate> {
        use tokio::process::Command;

        let output = if super::is_running_in_distrobox() {
            Command::new("distrobox-host-exec")
                .args(["rpm-ostree", "upgrade", "--check"])
                .output()
                .await
        } else {
            Command::new("rpm-ostree")
                .args(["upgrade", "--check"])
                .output()
                .await
        };

        let output = match output {
            Ok(output) => output,
            Err(_err) => return None,
        };

        let stdout = String::from_utf8_lossy(&output.stdout);

        // check if an update is available by looking for "AvailableUpdate:"
        if !stdout.contains("AvailableUpdate:") {
            return None;
        }

        let mut version = String::new();
        let mut commit = String::new();
        let mut gpg_signature = String::new();
        let mut sec_advisories = None;
        let mut diff = String::new();

        for line in stdout.lines() {
            let line = line.trim();

            if line.starts_with("Version:") {
                version = line
                    .strip_prefix("Version:")
                    .unwrap_or("")
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .to_string();
            } else if line.starts_with("Commit:") {
                commit = line
                    .strip_prefix("Commit:")
                    .unwrap_or("")
                    .trim()
                    .to_string();
            } else if line.starts_with("GPGSignature:") {
                let sig_info = line.strip_prefix("GPGSignature:").unwrap_or("").trim();

                gpg_signature = sig_info.to_string();
            } else if line.starts_with("SecAdvisories:") {
                let advisories = line.strip_prefix("SecAdvisories:").unwrap_or("").trim();

                if !advisories.is_empty() {
                    sec_advisories = Some(advisories.to_string());
                }
            } else if line.starts_with("Diff:") {
                diff = line.strip_prefix("Diff:").unwrap_or("").trim().to_string();
            }
        }

        // return parsed update if we have the required fields
        if !version.is_empty() && !commit.is_empty() {
            Some(SystemUpdate {
                version,
                commit,
                gpg_signature,
                sec_advisories,
                diff,
            })
        } else {
            None
        }
    }

    /// Update the system using: pkexec rpm-ostree upgrade
    pub async fn update() -> Result<(), anywho::Error> {
        use tokio::process::Command;

        let output = if super::is_running_in_distrobox() {
            Command::new("distrobox-host-exec")
                .args(["pkexec", "rpm-ostree", "upgrade"])
                .output()
                .await
        } else {
            Command::new("pkexec")
                .args(["rpm-ostree", "upgrade"])
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
}
