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

        let available_updates = match Self::get_available_updates().await {
            Ok(updates) => updates,
            Err(e) => {
                eprintln!("Failed to get available updates: {}", e);
                return Err(anywho!("Failed to get available updates: {}", e));
            }
        };

        for (app_id, latest_version) in available_updates {
            if let Some(current_version) = installed_apps.get(&app_id) {
                let icon_path = Self::get_app_icon(&app_id);
                let display_name = Self::get_app_display_name(&app_id)
                    .await
                    .unwrap_or(app_id.clone());

                applications.push(Application {
                    name: display_name,
                    app_id,
                    icon: icon_path,
                    current_version: current_version.to_string(),
                    latest_version,
                    application_status: ApplicationStatus::default(),
                });
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

    /// Returns the available updates that can actually be updated as HashMap<app_id, version>
    async fn get_available_updates() -> Result<HashMap<String, String>, anywho::Error> {
        use std::sync::Arc;
        use tokio::sync::Mutex;
        use tokio::task;

        println!("Checking for updates...");

        let updates = Arc::new(Mutex::new(HashMap::new()));
        let installations = vec!["--user", "--system"];
        let mut handles = Vec::new();

        for installation_flag in installations {
            let updates = updates.clone();
            let installation = installation_flag.to_string();

            let handle = task::spawn(async move {
                println!(
                    "Checking {} installation...",
                    if installation == "--user" {
                        "user"
                    } else {
                        "system"
                    }
                );

                // get all available updates with versions in one call
                let all_updates_task = task::spawn_blocking({
                    let installation = installation.clone();
                    move || {
                        Command::new("flatpak")
                            .args([
                                "remote-ls",
                                &installation,
                                "--updates",
                                "--app",
                                "--columns=application,version",
                            ])
                            .output()
                    }
                });

                // get the list of actually updatable apps
                let updatable_apps_task = task::spawn_blocking({
                    let installation = installation.clone();
                    move || {
                        Command::new("flatpak")
                            .args(["update", &installation])
                            .stdin(std::process::Stdio::piped())
                            .stdout(std::process::Stdio::piped())
                            .stderr(std::process::Stdio::piped())
                            .spawn()
                            .and_then(|mut child| {
                                // send 'n' to decline the update, so we just get the list
                                if let Some(mut stdin) = child.stdin.take() {
                                    use std::io::Write;
                                    let _ = stdin.write_all(b"n\n");
                                }
                                child.wait_with_output()
                            })
                    }
                });

                // Wait for both tasks to complete
                let (all_updates_result, updatable_apps_result) =
                    tokio::join!(all_updates_task, updatable_apps_task);

                // Parse available updates into a HashMap
                let mut available_versions = HashMap::new();
                #[allow(clippy::collapsible_if)]
                if let Ok(Ok(cmd_output)) = all_updates_result {
                    if cmd_output.status.success() {
                        let output_str = String::from_utf8_lossy(&cmd_output.stdout);
                        for line in output_str.lines() {
                            let parts: Vec<&str> = line.split_whitespace().collect();
                            if parts.len() >= 2 {
                                available_versions
                                    .insert(parts[0].to_string(), parts[1].to_string());
                            }
                        }
                    }
                }

                // parse updatable apps
                if let Ok(Ok(cmd_output)) = updatable_apps_result {
                    let output_str = String::from_utf8_lossy(&cmd_output.stdout);
                    let mut local_updates = HashMap::new();
                    let mut found_list = false;

                    for line in output_str.lines() {
                        let trimmed = line.trim();

                        // look for the start of the numbered list
                        if !found_list && trimmed.starts_with("1.") {
                            found_list = true;
                        }

                        // if we found the list, parse numbered entries
                        if found_list {
                            // parse lines like: "1.   org.gnome.Calculator stable  u   fedora  <   2,5 MB"
                            #[allow(clippy::collapsible_if)]
                            if let Some(number_end) = trimmed.find('.') {
                                if trimmed[..number_end]
                                    .chars()
                                    .all(|c| c.is_ascii_digit() || c.is_whitespace())
                                {
                                    let after_number = &trimmed[number_end + 1..].trim();
                                    let parts: Vec<&str> =
                                        after_number.split_whitespace().collect();

                                    if !parts.is_empty() {
                                        let app_id = parts[0];

                                        // look up version from pre-fetched map
                                        if let Some(version) = available_versions.get(app_id) {
                                            println!(
                                                "Found updatable app: {} -> {}",
                                                app_id, version
                                            );
                                            local_updates
                                                .insert(app_id.to_string(), version.clone());
                                        }
                                    }
                                }
                            }
                        }
                    }

                    let mut global_updates = updates.lock().await;
                    global_updates.extend(local_updates);
                }
            });

            handles.push(handle);
        }

        // wait for all installations to be checked
        for handle in handles {
            if let Err(e) = handle.await {
                eprintln!("Installation check task panicked: {:?}", e);
            }
        }

        let updates = Arc::try_unwrap(updates)
            .map_err(|_| anywho!("Failed to unwrap Arc"))?
            .into_inner();

        if !updates.is_empty() {
            println!("Found {} total updatable apps", updates.len());
            Ok(updates)
        } else {
            Err(anywho!("No updates found"))
        }
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
            .and_then(|path| {
                if path.extension().and_then(|ext| ext.to_str()) == Some("svg") {
                    Some(path.to_string_lossy().into_owned())
                } else {
                    None
                }
            })
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
