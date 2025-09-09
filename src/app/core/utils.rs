use anywho::anywho;

/// Check if a reboot is pending to apply staged updates
pub async fn reboot_pending() -> bool {
    use tokio::process::Command;

    let output = match Command::new("rpm-ostree").args(["status"]).output().await {
        Ok(output) => output,
        Err(_) => {
            match Command::new("distrobox-host-exec")
                .args(["rpm-ostree", "status"])
                .output()
                .await
            {
                Ok(output) => output,
                Err(_) => return false,
            }
        }
    };

    if !output.status.success() {
        return false;
    }

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
    use tokio::process::Command;

    let in_distrobox = std::env::var("DISTROBOX_ENTER_PATH").is_ok()
        || std::path::Path::new("/run/.containerenv").exists()
        || std::path::Path::new("/.dockerenv").exists();

    let output = if in_distrobox {
        Command::new("distrobox-host-exec")
            .args(["systemctl", "reboot"])
            .output()
            .await?
    } else {
        match Command::new("systemctl").args(["reboot"]).output().await {
            Ok(output) => output,
            Err(_) => {
                // fallback to distrobox-host-exec if systemctl fails
                Command::new("distrobox-host-exec")
                    .args(["systemctl", "reboot"])
                    .output()
                    .await?
            }
        }
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
