// SPDX-License-Identifier: GPL-3.0-only

use std::os::unix::process::ExitStatusExt;
use std::process::ExitStatus;

use anywho::anywho;

use crate::app::core::run_command;

/// Checks if a given package exists, returns the package name if succeeded
pub async fn check_package(package_name: String) -> Result<String, CheckPackageError> {
    let output = run_command("rpm-ostree", &["search", &package_name]).await;

    let output = match output {
        Ok(output) => output,
        Err(err) => {
            return Err(CheckPackageError::Error(anywho!(
                "Error searching for package: {}",
                err
            )));
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout);

    if stdout.contains("No matches found.") {
        return Err(CheckPackageError::NotFound);
    }

    for line in stdout.lines() {
        let line = line.trim();

        // skip empty lines and section headers
        if line.is_empty() || line.starts_with("=====") {
            continue;
        }

        // Parse package lines (format: "package-name : description")
        if let Some(colon_pos) = line.find(" : ") {
            let found_package_name = line[..colon_pos].trim();

            if found_package_name == package_name {
                return Ok(found_package_name.trim().to_string());
            }
        }
    }

    // if we get here, the search returned results but didn't contain an exact match
    Err(CheckPackageError::NotFound)
}

pub async fn add_packages(packages_to_add: Vec<String>) -> Result<(), anywho::Error> {
    let mut args: Vec<&str> = Vec::with_capacity(1 + packages_to_add.len());
    args.push("install");
    args.extend(packages_to_add.iter().map(|s| s.as_str()));

    let output = run_command("rpm-ostree", &args).await;

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

pub async fn remove_packages(packages_to_remove: Vec<String>) -> Result<(), anywho::Error> {
    let mut args: Vec<&str> = Vec::with_capacity(1 + packages_to_remove.len());
    args.push("uninstall");
    args.extend(packages_to_remove.iter().map(|s| s.as_str()));

    let output = run_command("rpm-ostree", &args).await;

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

#[derive(Debug, Clone)]
pub enum CheckPackageError {
    NotFound,
    Error(anywho::Error),
}

impl std::fmt::Display for CheckPackageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CheckPackageError::NotFound => write!(f, "Package Not Found"),
            CheckPackageError::Error(msg) => write!(f, "Update failed: {}", msg),
        }
    }
}

impl std::error::Error for CheckPackageError {}
