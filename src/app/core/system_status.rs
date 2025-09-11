use anywho::anywho;

#[derive(Debug, Clone)]
pub struct Deployment {
    pub name: String,
    pub version: String,
    pub base_commit: String,
    pub gpg_signature: String,
    pub layered_packages: String,
    pub is_pinned: bool,
    pub index: i32,
}

impl Deployment {
    /// Get's all the current deployments using rpm-ostree status
    pub async fn get_all() -> Result<Vec<Deployment>, anywho::Error> {
        use tokio::process::Command;

        // try direct rpm-ostree if not in distrobox
        let output = match Command::new("rpm-ostree").args(["status"]).output().await {
            Ok(output) => output,
            Err(_) => {
                // fallback: use distrobox-host-exec to run commands on the host system from within distrobox
                match Command::new("distrobox-host-exec")
                    .args(["rpm-ostree", "status"])
                    .output()
                    .await
                {
                    Ok(output) => output,
                    Err(_) => return Err(anywho!("Error fetching System Status")),
                }
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut deployments = Vec::new();
        let mut current_deployment: Option<Deployment> = None;
        let mut deployment_index = 0i32;

        for line in stdout.lines() {
            let line = line.trim();

            // skip empty lines and headers
            if line.is_empty() || line.starts_with("State:") || line.starts_with("Deployments:") {
                continue;
            }

            // new deployment starts with ● or ○
            if line.starts_with('●') || line.starts_with('○') {
                // save previous deployment if exists
                if let Some(deployment) = current_deployment.take() {
                    deployments.push(deployment);
                }

                // parse the deployment line: "● fedora:fedora/42/x86_64/silverblue"
                let parts: Vec<&str> = line.splitn(2, ' ').collect();
                if parts.len() >= 2 {
                    let name_version = parts[1].trim();
                    let (name, version) = if let Some(colon_pos) = name_version.find(':') {
                        let name = name_version[..colon_pos].to_string();
                        let version_part = &name_version[colon_pos + 1..];
                        (name, version_part.to_string())
                    } else {
                        (name_version.to_string(), String::new())
                    };

                    current_deployment = Some(Deployment {
                        name,
                        version,
                        base_commit: String::new(),
                        gpg_signature: String::new(),
                        layered_packages: String::new(),
                        is_pinned: false,
                        index: deployment_index,
                    });

                    deployment_index += 1;
                }
            }
            // parse fields for current deployment
            else if let Some(ref mut deployment) = current_deployment {
                if line.starts_with("BaseCommit:") {
                    deployment.base_commit =
                        line.trim_start_matches("BaseCommit:").trim().to_string();
                } else if line.starts_with("GPGSignature:") {
                    deployment.gpg_signature =
                        line.trim_start_matches("GPGSignature:").trim().to_string();
                } else if line.starts_with("LayeredPackages:") {
                    deployment.layered_packages = line
                        .trim_start_matches("LayeredPackages:")
                        .trim()
                        .to_string();

                    if deployment.layered_packages.is_empty() {
                        deployment.layered_packages = String::from("None");
                    }
                } else if line.starts_with("Pinned:") {
                    #[allow(clippy::collapsible_if)]
                    if line
                        .trim_start_matches("Pinned:")
                        .trim()
                        .to_string()
                        .eq_ignore_ascii_case("yes")
                    {
                        deployment.is_pinned = true;
                    }
                }
            }
        }

        // add the last deployment
        if let Some(deployment) = current_deployment {
            deployments.push(deployment);
        }

        if deployments.is_empty() {
            return Err(anywho!("No deployments found in rpm-ostree status output"));
        }

        Ok(deployments)
    }

    pub async fn pin_deployment(deployment_index: i32) -> Result<(), anywho::Error> {
        use tokio::process::Command;

        // pkexec first
        let output = Command::new("pkexec")
            .args(["rpm-ostree", "admin", "pin", &deployment_index.to_string()])
            .output()
            .await?;

        if output.status.success() {
            return Ok(());
        }

        // fallback to distrobox-host-exec
        let output = Command::new("distrobox-host-exec")
            .args([
                "pkexec",
                "rpm-ostree",
                "admin",
                "pin",
                &deployment_index.to_string(),
            ])
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

    pub async fn unpin_deployment(deployment_index: i32) -> Result<(), anywho::Error> {
        use tokio::process::Command;

        // pkexec first
        let output = Command::new("pkexec")
            .args([
                "rpm-ostree",
                "admin",
                "pin",
                "--unpin",
                &deployment_index.to_string(),
            ])
            .output()
            .await?;

        if output.status.success() {
            return Ok(());
        }

        // fallback to distrobox-host-exec
        let output = Command::new("distrobox-host-exec")
            .args([
                "pkexec",
                "rpm-ostree",
                "admin",
                "pin",
                "--unpin",
                &deployment_index.to_string(),
            ])
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
}
