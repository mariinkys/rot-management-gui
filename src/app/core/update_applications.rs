use anywho::anywho;
use std::collections::HashMap;

use crate::app::core::run_command;

#[derive(Debug, Clone)]
pub struct Application {
    pub name: String,
    pub app_id: String,
    pub icon: Option<AppIcon>,
    pub latest_version: String,
    pub application_status: ApplicationStatus,
}

#[derive(Debug, Clone)]
pub enum AppIcon {
    Svg { path: String },
    Image { path: String },
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum ApplicationStatus {
    Updating,
    #[default]
    NotUpdating,
}

/// Needed for internal (update_applications) usage
#[derive(Debug)]
struct AppInfo {
    ref_name: String,
    origin: String,
}

impl Application {
    /// Returns a Vector of all [`Application`] that have available updates
    pub async fn get_all_available_updates() -> Result<Vec<Application>, anywho::Error> {
        let mut applications = Vec::new();

        let available_updates = match Self::get_available_updates().await {
            Ok(updates) => updates,
            Err(e) => {
                eprintln!("Failed to get available updates: {}", e);
                return Err(anywho!("Failed to get available updates: {}", e));
            }
        };

        for (app_id, latest_version) in available_updates {
            let icon_path = Self::get_app_icon(&app_id);
            let display_name = Self::get_app_display_name(&app_id)
                .await
                .unwrap_or(app_id.clone());

            applications.push(Application {
                name: display_name,
                app_id,
                icon: icon_path,
                latest_version,
                application_status: ApplicationStatus::default(),
            });
        }

        Ok(applications)
    }

    /// Returns the available updates that can actually be updated as HashMap<app_id, version>
    async fn get_available_updates() -> Result<HashMap<String, String>, anywho::Error> {
        use futures::future::join_all;
        use std::collections::HashMap;

        println!("Checking for updates...");
        let installations = vec!["--user", "--system"];
        let mut handles = Vec::new();

        for &installation in &installations {
            let installation_type = if installation == "--user" {
                "user"
            } else {
                "system"
            };
            println!("Checking {} installation for updates...", installation_type);

            let handle = tokio::spawn(async move {
                // get installed apps with versions and origins
                let installed_apps = match Self::get_installed_apps(installation).await {
                    Ok(apps) => apps,
                    Err(e) => {
                        eprintln!(
                            "Failed to get installed apps for {}: {}",
                            installation_type, e
                        );
                        return HashMap::new();
                    }
                };

                if installed_apps.is_empty() {
                    return HashMap::new();
                }

                // get remote versions for installed apps
                match Self::get_remote_versions(&installed_apps, installation).await {
                    Ok(remote_versions) => {
                        let mut updates = HashMap::new();

                        // Create a mapping of normalized ref names to app IDs
                        let mut app_id_by_ref = HashMap::new();
                        for (app_id, app_info) in &installed_apps {
                            // Store both normalized and original ref formats
                            let normalized_ref = app_info
                                .ref_name
                                .strip_prefix("app/")
                                .unwrap_or(&app_info.ref_name)
                                .to_string();
                            app_id_by_ref.insert(app_info.ref_name.clone(), app_id.clone());
                            app_id_by_ref.insert(normalized_ref, app_id.clone());
                        }

                        // For each remote version, find the corresponding app ID
                        for (remote_ref, version) in remote_versions {
                            if let Some(app_id) = app_id_by_ref.get(&remote_ref) {
                                updates.insert(app_id.clone(), version);
                            } else if let Some(app_id) =
                                app_id_by_ref.get(&format!("app/{}", remote_ref))
                            {
                                updates.insert(app_id.clone(), version);
                            } else if let Some(app_id) = Self::extract_app_id_from_ref(&remote_ref)
                            {
                                // Final fallback: try to match by app ID
                                if installed_apps.contains_key(&app_id) {
                                    updates.insert(app_id, version);
                                }
                            }
                        }

                        updates
                    }
                    Err(e) => {
                        eprintln!(
                            "Failed to get remote versions for {}: {}",
                            installation_type, e
                        );
                        HashMap::new()
                    }
                }
            });

            handles.push(handle);
        }

        let results = join_all(handles).await;
        let mut all_updates = HashMap::new();
        for updates in results.into_iter().flatten() {
            all_updates.extend(updates);
        }

        if all_updates.is_empty() {
            Err(anywho!("No updates found"))
        } else {
            println!("Found {} total updatable apps", all_updates.len());
            Ok(all_updates)
        }
    }

    /// Get installed apps with their ref, and origin remote
    async fn get_installed_apps(
        installation: &str,
    ) -> Result<HashMap<String, AppInfo>, anywho::Error> {
        let args = vec![
            "list",
            installation,
            "--app",
            "--columns=application,version,origin,ref",
        ];

        let output = super::run_command("flatpak", &args)
            .await
            .map_err(|e| anywho!("Failed to list installed apps: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anywho!("Failed to list apps: {}", stderr.trim()));
        }

        let mut apps = HashMap::new();
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 4 {
                let app_id = parts[0].trim().to_string();
                // let version = parts[1].trim().to_string();
                let origin = parts[2].trim().to_string();
                let ref_name = parts[3].trim().to_string();

                apps.insert(app_id, AppInfo { ref_name, origin });
            }
        }
        Ok(apps)
    }

    /// Get remote versions for installed apps grouped by origin remote
    async fn get_remote_versions(
        installed_apps: &HashMap<String, AppInfo>,
        installation: &str,
    ) -> Result<HashMap<String, String>, anywho::Error> {
        let mut remotes: HashMap<String, Vec<String>> = HashMap::new();
        for app in installed_apps.values() {
            remotes
                .entry(app.origin.clone())
                .or_default()
                .push(app.ref_name.clone());
        }

        let mut remote_versions = HashMap::new();

        for (remote, refs) in remotes {
            if refs.is_empty() {
                continue;
            }

            println!(
                "Getting remote versions from {} for {} refs",
                remote,
                refs.len()
            );

            let args = vec![
                "remote-ls",
                installation,
                "--updates",
                "--app",
                "--columns=ref,version",
                &remote,
            ];

            let output = super::run_command("flatpak", &args)
                .await
                .map_err(|e| anywho!("Failed to get remote info from {}: {}", remote, e))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!(
                    "Warning: remote-ls failed for {}: {}",
                    remote,
                    stderr.trim()
                );
                continue;
            }

            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                // Split by tab character
                let parts: Vec<&str> = trimmed.split('\t').collect();
                if parts.len() >= 2 {
                    let ref_name = parts[0].trim().to_string();
                    let version = parts[1].trim().to_string();
                    remote_versions.insert(ref_name, version);
                }
            }
        }

        println!("Found {} remote versions", remote_versions.len());
        Ok(remote_versions)
    }

    /// Extract app ID from a ref name
    fn extract_app_id_from_ref(ref_name: &str) -> Option<String> {
        let parts: Vec<&str> = ref_name.split('/').collect();

        // Handle all possible ref formats:
        // - app/org.fedoraproject.MediaWriter/x86_64/stable
        // - runtime/org.kde.Platform/x86_64/6.9
        // - org.fedoraproject.MediaWriter/x86_64/stable
        // - extension/org.gnome.Shell.Extensions/x86_64/stable

        match parts.len() {
            4 => {
                // Format: type/app_id/arch/branch
                if parts[0] == "app" || parts[0] == "runtime" || parts[0] == "extension" {
                    Some(parts[1].to_string())
                } else {
                    // Format: app_id/arch/branch/something (unlikely but handle it)
                    Some(parts[0].to_string())
                }
            }
            3 => {
                // Format: app_id/arch/branch
                Some(parts[0].to_string())
            }
            _ => {
                // Fallback: try to find the longest part that looks like a domain name
                parts
                    .into_iter()
                    .find(|p| p.contains('.'))
                    .map(|s| s.to_string())
            }
        }
    }

    /// Get the display name for an application
    async fn get_app_display_name(app_id: &str) -> Result<String, anywho::Error> {
        let output = run_command("flatpak", &["info", "--show-metadata", app_id]).await?;

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

    fn get_app_icon(app_id: &str) -> Option<AppIcon> {
        freedesktop_icons::lookup(app_id)
            .force_svg()
            .with_cache()
            .with_size(256)
            .find()
            .and_then(|path| {
                if let Some(ext) = path.extension().and_then(|ext| ext.to_str()) {
                    match ext {
                        "svg" => Some(AppIcon::Svg {
                            path: path.to_string_lossy().into_owned(),
                        }),
                        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "ico" | "tiff" => {
                            Some(AppIcon::Image {
                                path: path.to_string_lossy().into_owned(),
                            })
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            })
    }

    /// Update a specific application
    pub async fn update(app_id: String) -> Result<(), UpdateError> {
        let output = run_command("flatpak", &["update", "-y", &app_id])
            .await
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

        let output = run_command("flatpak", &["update", "-y"])
            .await
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
