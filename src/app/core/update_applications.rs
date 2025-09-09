// TODO: Explore using tokio::Command instead of the std:: one

use anywho::anywho;
use std::collections::HashMap;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct Application {
    pub name: String,
    pub app_id: String,
    pub icon: Option<String>,
    pub current_version: String,
    pub latest_version: String,
    pub application_status: ApplicationStatus,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum ApplicationStatus {
    Updating,
    #[default]
    NotUpdating,
}

impl Application {
    /// Returns a Vector of all [`Application`] that have available updates
    pub async fn get_all_available_updates() -> Result<Vec<Application>, anywho::Error> {
        let mut applications = Vec::new();

        let installed_apps = match Self::get_installed_applications().await {
            Ok(apps) => apps,
            Err(e) => {
                eprintln!("Failed to get installed applications: {}", e);
                return Err(anywho!("Failed to get installed applications: {}", e));
            }
        };

        let available_updates = match Self::get_available_updates(&installed_apps).await {
            Ok(updates) => updates,
            Err(e) => {
                eprintln!("Failed to get available updates: {}", e);
                return Err(anywho!("Failed to get available updates: {}", e));
            }
        };

        for (app_id, current_version) in installed_apps {
            #[allow(clippy::collapsible_if)]
            if let Some(latest_version) = available_updates.get(&app_id) {
                if current_version != *latest_version {
                    let icon_path = Self::get_app_icon(&app_id);
                    let display_name = Self::get_app_display_name(&app_id)
                        .await
                        .unwrap_or(app_id.clone());

                    applications.push(Application {
                        name: display_name,
                        app_id,
                        icon: icon_path,
                        current_version,
                        latest_version: latest_version.clone(),
                        application_status: ApplicationStatus::default(),
                    });
                }
            }
        }

        Ok(applications)
    }

    /// Get all installed Flatpak applications with their versions
    async fn get_installed_applications() -> Result<HashMap<String, String>, anywho::Error> {
        let output = Command::new("flatpak")
            .args(["list", "--app", "--columns=application,version"])
            .output()?;

        if !output.status.success() {
            return Err(anywho!(
                "flatpak list failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let mut apps = HashMap::new();
        let output_str = String::from_utf8_lossy(&output.stdout);

        for line in output_str.lines().skip(1) {
            // Skip header
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 2 {
                let app_id = parts[0].trim().to_string();
                let version = parts[1].trim().to_string();
                apps.insert(app_id, version);
            }
        }

        Ok(apps)
    }

    async fn get_available_updates(
        installed_apps: &HashMap<String, String>,
    ) -> Result<HashMap<String, String>, anywho::Error> {
        use std::sync::Arc;
        use tokio::sync::Semaphore;

        println!("Checking for updates...");
        let appstream_update = Command::new("flatpak")
            .args(["update", "--appstream"])
            .output();
        if let Err(e) = appstream_update {
            eprintln!("Warning: Failed to update appstream data: {}", e);
        }

        let updates = Arc::new(tokio::sync::Mutex::new(HashMap::new()));

        let semaphore = Arc::new(Semaphore::new(20));
        let mut handles = Vec::new();

        for (app_id, current_version) in installed_apps.iter() {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let updates = updates.clone();
            let app_id = app_id.clone();
            let current_version = current_version.clone();

            let handle = tokio::task::spawn(async move {
                let _permit = permit;

                #[allow(clippy::collapsible_if)]
                if let Ok(remote_version) = Self::get_remote_version(&app_id).await {
                    if Self::is_version_newer(&remote_version, &current_version) {
                        println!(
                            "Found update for {}: {} -> {}",
                            app_id, current_version, remote_version
                        );
                        updates.lock().await.insert(app_id, remote_version);
                    }
                }
            });

            handles.push(handle);
        }

        // wait for all tasks to finish
        for handle in handles {
            let _ = handle.await;
        }

        let updates = Arc::try_unwrap(updates).unwrap().into_inner();

        if !updates.is_empty() {
            println!("Found {} updates", updates.len());
            Ok(updates)
        } else {
            Err(anywho!("No updates found"))
        }
    }

    /// Get the remote version of a specific application
    async fn get_remote_version(app_id: &str) -> Result<String, anywho::Error> {
        let output = Command::new("flatpak")
            .args(["remote-info", "flathub", app_id])
            .output()?;

        if !output.status.success() {
            return Err(anywho!("Failed to get remote info for {}", app_id));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);

        for line in output_str.lines() {
            let line = line.trim_start();
            if let Some(version) = line.strip_prefix("Version:") {
                return Ok(version.trim().to_string());
            }
        }

        for line in output_str.lines() {
            let line = line.trim_start();
            if let Some(commit) = line.strip_prefix("Commit:") {
                return Ok(commit.trim().to_string());
            }
        }

        Err(anywho!("No version information found for {}", app_id))
    }

    /// Compare two version strings to determine if the remote version is newer
    fn is_version_newer(remote_version: &str, current_version: &str) -> bool {
        if remote_version == current_version {
            return false;
        }

        let normalize_version = |v: &str| -> Vec<String> {
            v.split(&['.', '-', '_'][..])
                .map(|s| s.to_string())
                .collect()
        };

        let remote_parts = normalize_version(remote_version);
        let current_parts = normalize_version(current_version);

        let max_len = remote_parts.len().max(current_parts.len());

        for i in 0..max_len {
            let default_val = String::from("0");

            let remote_part = remote_parts.get(i).unwrap_or(&default_val);
            let current_part = current_parts.get(i).unwrap_or(&default_val);

            // Try to parse as numbers first
            match (remote_part.parse::<u64>(), current_part.parse::<u64>()) {
                (Ok(remote_num), Ok(current_num)) => {
                    if remote_num > current_num {
                        return true;
                    } else if remote_num < current_num {
                        return false;
                    }
                }
                _ => match remote_part.cmp(current_part) {
                    std::cmp::Ordering::Greater => return true,
                    std::cmp::Ordering::Less => return false,
                    std::cmp::Ordering::Equal => continue,
                },
            }
        }

        // if all parts are equal, check if remote has more parts
        remote_parts.len() > current_parts.len()
    }

    /// Get the display name for an application
    async fn get_app_display_name(app_id: &str) -> Result<String, anywho::Error> {
        let output = Command::new("flatpak")
            .args(["info", "--show-metadata", app_id])
            .output()?;

        if !output.status.success() {
            return Ok(app_id.to_string()); // fallback to app ID
        }

        let output_str = String::from_utf8_lossy(&output.stdout);

        for line in output_str.lines() {
            if line.starts_with("[Application]") {
                continue;
            }
            if line.starts_with("name=") {
                return Ok(line.strip_prefix("name=").unwrap_or(app_id).to_string());
            }
        }

        Ok(app_id.to_string())
    }

    fn get_app_icon(app_id: &str) -> Option<String> {
        freedesktop_icons::lookup(app_id)
            .force_svg()
            .with_cache()
            .find()
            .map(|path| path.to_string_lossy().into_owned())
    }

    /// Update a specific application
    pub async fn update(app_id: String) -> Result<(), UpdateError> {
        let output = Command::new("flatpak")
            .args(["update", "-y", &app_id])
            .output()
            .map_err(|e| UpdateError::CommandFailed(anywho!("{}", e)))?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(UpdateError::UpdateFailed(format!(
                "Failed to update {}: {}",
                app_id, error_msg
            )));
        }

        println!("Successfully updated: {}", app_id);
        Ok(())
    }

    /// Update all applications with available updates
    pub async fn update_all(
        apps_to_update: Vec<Application>,
    ) -> Result<Vec<UpdateResult>, UpdateError> {
        let mut results = Vec::new();

        // update all at once
        let output = Command::new("flatpak")
            .args(["update", "-y"])
            .output()
            .map_err(|e| UpdateError::CommandFailed(anywho!("{}", e)))?;

        if output.status.success() {
            for app in apps_to_update {
                results.push(UpdateResult {
                    app_name: app.name,
                    success: true,
                    error_message: None,
                });
            }
            println!("Successfully updated all applications");
        } else {
            // if bulk update fails, try individual updates
            for app in apps_to_update {
                match Self::update(app.app_id).await {
                    Ok(()) => results.push(UpdateResult {
                        app_name: app.name,
                        success: true,
                        error_message: None,
                    }),
                    Err(e) => results.push(UpdateResult {
                        app_name: app.name,
                        success: false,
                        error_message: Some(e.to_string()),
                    }),
                }
            }
        }

        Ok(results)
    }
}

/// Represents the result of an update operation
#[derive(Debug, Clone)]
pub struct UpdateResult {
    pub app_name: String,
    pub success: bool,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub enum UpdateError {
    CommandFailed(anywho::Error),
    UpdateFailed(String),
}

impl std::fmt::Display for UpdateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpdateError::CommandFailed(e) => write!(f, "Command execution failed: {}", e),
            UpdateError::UpdateFailed(msg) => write!(f, "Update failed: {}", msg),
        }
    }
}

impl std::error::Error for UpdateError {}
