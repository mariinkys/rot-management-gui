use anywho::anywho;
use std::os::unix::process::ExitStatusExt;
use std::process::ExitStatus;

use crate::app::core::run_command;

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
        let output = run_command("rpm-ostree", &["status"]).await;

        let output = match output {
            Ok(output) => output,
            Err(err) => return Err(anywho!("Error fetching System Status: {}", err)),
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut deployments = Vec::new();
        let mut current_deployment: Option<Deployment> = None;
        let mut deployment_index = 0i32;
        let mut building_deployment_name = String::new();

        for line in stdout.lines() {
            let line = line.trim();

            // skip empty lines and headers
            if line.is_empty() || line.starts_with("State:") || line.starts_with("Deployments:") {
                continue;
            }

            // check if this is the start of a new deployment (● or ○ or deployment without bullet)
            if line.starts_with('●')
                || line.starts_with('○')
                || (Self::is_deployment_line(line) && current_deployment.is_none())
            {
                // save previous deployment if exists
                if let Some(deployment) = current_deployment.take() {
                    deployments.push(deployment);
                }

                // extract deployment name
                let deployment_line = if line.starts_with('●') || line.starts_with('○') {
                    line.chars().skip(1).collect::<String>().trim().to_string()
                } else {
                    line.to_string()
                };

                // check if the deployment name continues on the next line (like happens in Bazzite...)
                if deployment_line.is_empty() {
                    building_deployment_name.clear();
                    current_deployment = Some(Deployment {
                        name: String::new(),
                        version: String::new(),
                        base_commit: String::new(),
                        gpg_signature: String::new(),
                        layered_packages: String::new(),
                        is_pinned: false,
                        index: deployment_index,
                    });
                    deployment_index += 1;
                    continue;
                }

                // parse complete deployment name
                let (name, version) = Self::parse_deployment_name(&deployment_line);
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
            // check if this is a continuation of deployment name (indented, no field prefix)
            else if current_deployment.is_some()
                && line.starts_with(' ')
                && !line.contains(':')
                && !line.starts_with("Version:")
                && !line.starts_with("BaseCommit:")
                && !line.starts_with("Digest:")
                && !line.starts_with("GPGSignature:")
                && !line.starts_with("LayeredPackages:")
                && !line.starts_with("Pinned:")
            {
                // this is likely a continuation of the deployment name
                building_deployment_name.push_str(line.trim());

                // update the deployment name if we have one building
                if let Some(ref mut deployment) = current_deployment {
                    if deployment.name.is_empty() && !building_deployment_name.is_empty() {
                        let (name, version) =
                            Self::parse_deployment_name(&building_deployment_name);
                        deployment.name = name;
                        deployment.version = version;
                        building_deployment_name.clear();
                    }
                }
            }
            // check if this could be a deployment line without bullet (not indented, contains colon or slash)
            else if Self::is_deployment_line(line) && current_deployment.is_some() {
                // save previous deployment if exists
                if let Some(deployment) = current_deployment.take() {
                    deployments.push(deployment);
                }

                // parse this as a new deployment
                let (name, version) = Self::parse_deployment_name(line);
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
            // parse fields for current deployment
            else if let Some(ref mut deployment) = current_deployment {
                if line.starts_with("Version:") {
                    let version_info = line.trim_start_matches("Version:").trim();
                    // extract version number, ignore timestamp in parentheses
                    if let Some(space_pos) = version_info.find(' ') {
                        let version_num = &version_info[..space_pos];
                        if !deployment.version.is_empty() {
                            deployment.version = format!("{}/{}", deployment.version, version_num);
                        } else {
                            deployment.version = version_num.to_string();
                        }
                    } else {
                        if !deployment.version.is_empty() {
                            deployment.version = format!("{}/{}", deployment.version, version_info);
                        } else {
                            deployment.version = version_info.to_string();
                        }
                    }
                } else if line.starts_with("BaseCommit:") {
                    deployment.base_commit =
                        line.trim_start_matches("BaseCommit:").trim().to_string();
                } else if line.starts_with("Digest:") {
                    // Bazzite uses Digest instead of BaseCommit
                    deployment.base_commit = line.trim_start_matches("Digest:").trim().to_string();
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
                    let pinned_value = line.trim_start_matches("Pinned:").trim();
                    deployment.is_pinned = pinned_value.eq_ignore_ascii_case("yes");
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

    /// Helper function to detect if a line looks like a deployment name
    fn is_deployment_line(line: &str) -> bool {
        let trimmed = line.trim();

        // skip obvious field lines
        if trimmed.starts_with("Version:")
            || trimmed.starts_with("BaseCommit:")
            || trimmed.starts_with("Digest:")
            || trimmed.starts_with("GPGSignature:")
            || trimmed.starts_with("LayeredPackages:")
            || trimmed.starts_with("Pinned:")
        {
            return false;
        }

        // look for deployment patterns:
        // Contains colon (like "fedora:fedora/42/x86_64/silverblue" or "ostree-image-signed:docker://...")
        // Contains slash without colon (like "cs9/aarch64/rpi4-qa" or "local:xxx/7/x86_64/standard")
        // starts at beginning of line (not heavily indented)

        if trimmed.contains(':') {
            // has colon - likely a deployment ref
            return true;
        }

        if trimmed.contains('/') && !trimmed.starts_with("    ") {
            // has slash and not heavily indented - could be a deployment path
            return true;
        }

        // if it's a single word without special chars at start of line, could be simple deployment name
        if !trimmed.contains(' ') && !trimmed.starts_with("  ") && trimmed.len() > 3 {
            return true;
        }

        false
    }

    /// helper function to parse deployment name and extract name/version
    fn parse_deployment_name(deployment_line: &str) -> (String, String) {
        if let Some(colon_pos) = deployment_line.find(':') {
            let name = deployment_line[..colon_pos].to_string();
            let version_part = &deployment_line[colon_pos + 1..];
            (name, version_part.to_string())
        } else {
            (deployment_line.to_string(), String::new())
        }
    }

    pub async fn pin_deployment(deployment_index: i32) -> Result<(), anywho::Error> {
        let output = run_command(
            "pkexec",
            &["ostree", "admin", "pin", &deployment_index.to_string()],
        )
        .await;

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

    pub async fn unpin_deployment(deployment_index: i32) -> Result<(), anywho::Error> {
        let output = run_command(
            "pkexec",
            &[
                "ostree",
                "admin",
                "pin",
                "--unpin",
                &deployment_index.to_string(),
            ],
        )
        .await;

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
