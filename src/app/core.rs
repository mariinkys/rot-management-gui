// SPDX-License-Identifier: GPL-3.0-only

use anywho::anywho;

pub mod config;
pub mod rollback;
pub mod system_status;
pub mod update_applications;
pub mod update_system;

/// Runs the given command correctly for each possible context (Flatpak/Distrobox/System)
pub async fn run_command(
    main_command: &str,
    args: &[&str],
) -> Result<std::process::Output, std::io::Error> {
    use tokio::process::Command;

    if is_flatpak() {
        // If is flatpak we need to add flatpak-spawn --host
        Command::new("flatpak-spawn")
            .args(
                vec!["--host", main_command]
                    .into_iter()
                    .chain(args.iter().cloned()),
            )
            .output()
            .await
    } else if is_running_in_distrobox() {
        // If is distrobox we need to add distrobox-host-exec
        Command::new("distrobox-host-exec")
            .args(vec![main_command].into_iter().chain(args.iter().cloned()))
            .output()
            .await
    } else {
        // Add nothing (Ej: main_command: "pkexec", args: ["rpm-ostree", "rollback"])
        Command::new(main_command).args(args).output().await
    }
}

/// Checks if the application is running inside a flatpak
fn is_flatpak() -> bool {
    std::env::var("FLATPAK_ID").is_ok()
}

/// Checks if the application is running inside a toolbox/distrobox container
pub fn is_running_in_distrobox() -> bool {
    use std::env;
    use std::fs;

    // if env::var("TOOLBOX_PATH").is_ok() {
    //     return true;
    // }

    if env::var("DISTROBOX_ENTER").is_ok() {
        return true;
    }

    if let Ok(val) = env::var("container")
        && (val == "distrobox")
    // || val == "toolbox"
    {
        return true;
    }

    // Check special marker files
    let markers = ["/run/.containerenv", "/run/.toolboxenv"];
    if markers.iter().any(|path| fs::metadata(path).is_ok()) {
        return true;
    }

    false
}

/// Check if a reboot is pending to apply staged updates
pub async fn reboot_pending() -> bool {
    let output = run_command("rpm-ostree", &["status"]).await;

    // TODO: Maybe we should return an error here? not a bool
    let output = match output {
        Ok(output) => output,
        Err(_err) => return false,
    };

    let stdout = String::from_utf8_lossy(&output.stdout);

    // look for indicators of a pending deployment:
    // 1. Multiple deployments where the first one is not the booted one (marked with ●)
    // 2. A deployment that's staged/pending
    let lines: Vec<&str> = stdout.lines().collect();
    let mut found_deployments_section = false;
    let mut deployment_count = 0;
    let mut booted_deployment_index = None;

    for line in lines.iter() {
        let trimmed = line.trim();

        // find the Deployments section
        if trimmed.starts_with("Deployments:") {
            found_deployments_section = true;
            continue;
        }

        if found_deployments_section && !trimmed.is_empty() {
            // check if this line describes a deployment (starts with ●, *, or space + ostree://)
            if trimmed.starts_with("●") {
                // This is the currently booted deployment
                booted_deployment_index = Some(deployment_count);
                deployment_count += 1;
            } else if trimmed.starts_with("*")
                || (trimmed.starts_with("ostree://")
                    || trimmed.starts_with("fedora:")
                    || trimmed.starts_with("Version:"))
            {
                // this is a staged/pending deployment
                deployment_count += 1;
            } else if trimmed.starts_with("State:") || trimmed.starts_with("AutomaticUpdates:") {
                // we've moved past the deployments section
                break;
            }
        }
    }

    // if we have more than 1 deployment and the booted one is not the first (index 0),
    // then there's a pending deployment that requires reboot
    deployment_count > 1 && booted_deployment_index.unwrap_or(0) != 0
}

/// Reboot the system using: systemctl reboot
pub async fn reboot() -> Result<(), anywho::Error> {
    let output = run_command("systemctl", &["reboot"]).await;

    let output = match output {
        Ok(output) => output,
        Err(err) => return Err(anywho!("{}", err)),
    };

    // if reboot is successful, this code may never be reached
    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let error_msg = if !stderr.is_empty() {
            format!("systemctl reboot failed: {}", stderr.trim())
        } else if !stdout.is_empty() {
            format!("systemctl reboot failed: {}", stdout.trim())
        } else {
            "systemctl reboot failed with unknown error".to_string()
        };
        Err(anywho!("{}", error_msg))
    }
}
