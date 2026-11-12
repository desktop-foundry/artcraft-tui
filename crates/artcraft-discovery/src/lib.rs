use artcraft_domain::{
    Application, CpuArch, DetectionConfidence, HealthStatus, Installation, InstallationMethod,
    InstallationScope,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn cmd_output(bin: &str, args: &[&str]) -> Option<String> {
    Command::new(bin)
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
}

fn parse_arch(s: &str) -> CpuArch {
    match s.to_lowercase().as_str() {
        "amd64" | "x86_64" | "x86-64" => CpuArch::X86_64,
        "arm64" | "aarch64" => CpuArch::Aarch64,
        "riscv64" => CpuArch::Riscv64,
        _ => CpuArch::Unknown,
    }
}

fn app_for_package<'a>(apps: &'a [Application], pkg: &str) -> Option<&'a Application> {
    apps.iter()
        .find(|a| a.known_package_ids.iter().any(|p| p == pkg))
}

fn app_for_flatpak<'a>(apps: &'a [Application], fid: &str) -> Option<&'a Application> {
    apps.iter()
        .find(|a| a.known_package_ids.iter().any(|p| p == fid))
}

pub fn discover_all(apps: &[Application]) -> Vec<Installation> {
    let mut out = Vec::new();
    discover_dpkg(apps, &mut out);
    discover_flatpak(apps, &mut out);
    discover_appimages(apps, &mut out);
    discover_path(apps, &mut out);
    discover_desktop_files(apps, &mut out);
    out
}

fn discover_dpkg(apps: &[Application], out: &mut Vec<Installation>) {
    for app in apps {
        for pkg in &app.known_package_ids {
            // dpkg-query
            let o = Command::new("dpkg-query")
                .args(["-W", "-f=${Status}\t${Version}\t${Architecture}\n", pkg])
                .output();
            if let Ok(o) = o {
                if o.status.success() {
                    let text = String::from_utf8_lossy(&o.stdout);
                    for line in text.lines() {
                        let parts: Vec<&str> = line.split('\t').collect();
                        if parts.len() >= 3 && parts[0].contains("installed") {
                            out.push(Installation {
                                id: format!("dpkg:{}", pkg),
                                app_id: app.id.clone(),
                                installed_version: Some(parts[1].to_string()),
                                method: InstallationMethod::DebianPackage,
                                scope: InstallationScope::System,
                                executable_path: None,
                                package_identifier: Some(pkg.clone()),
                                package_manager: Some("dpkg".into()),
                                architecture: parse_arch(parts[2]),
                                installation_root: None,
                                provenance: Some("dpkg-query".into()),
                                health: HealthStatus::Unknown,
                                confidence: DetectionConfidence::High,
                                last_verified_at: None,
                            });
                        }
                    }
                }
            }
        }
    }
    // also apt origin? skip
    let _ = app_for_package;
}

fn discover_flatpak(apps: &[Application], out: &mut Vec<Installation>) {
    // flatpak list --app --columns=application,version,installation
    let text = match cmd_output(
        "flatpak",
        &[
            "list",
            "--app",
            "--columns=application,version,installation",
        ],
    ) {
        Some(t) => t,
        None => return,
    };
    for line in text.lines() {
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.is_empty() {
            continue;
        }
        let fid = cols[0].trim();
        if let Some(app) = app_for_flatpak(apps, fid) {
            let version = cols
                .get(1)
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());
            let install = cols
                .get(2)
                .map(|s| s.trim().to_lowercase())
                .unwrap_or_default();
            let (method, scope) = if install.contains("system") {
                (InstallationMethod::FlatpakSystem, InstallationScope::System)
            } else {
                (InstallationMethod::FlatpakUser, InstallationScope::User)
            };
            out.push(Installation {
                id: format!("flatpak:{}", fid),
                app_id: app.id.clone(),
                installed_version: version,
                method,
                scope,
                executable_path: None,
                package_identifier: Some(fid.to_string()),
                package_manager: Some("flatpak".into()),
                architecture: CpuArch::Unknown,
                installation_root: None,
                provenance: Some("flatpak list".into()),
                health: HealthStatus::Unknown,
                confidence: DetectionConfidence::High,
                last_verified_at: None,
            });
        }
    }
}

fn discover_appimages(apps: &[Application], out: &mut Vec<Installation>) {
    let mut dirs = Vec::new();
    if let Some(home) = std::env::var_os("HOME") {
        let home = PathBuf::from(home);
        dirs.push(home.join("Applications"));
        dirs.push(home.join(".local/bin"));
    }
    dirs.push(PathBuf::from("/opt"));
    for dir in dirs {
        let entries = match fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if !name.ends_with(".AppImage") {
                continue;
            }
            // try to match app by filename
            let lower = name.to_lowercase();
            for app in apps {
                let matches = app
                    .executable_names
                    .iter()
                    .any(|n| lower.contains(&n.to_lowercase()))
                    || app
                        .display_name
                        .to_lowercase()
                        .split_whitespace()
                        .any(|w| lower.contains(w));
                if matches {
                    out.push(Installation {
                        id: format!("appimage:{}", path.display()),
                        app_id: app.id.clone(),
                        installed_version: None,
                        method: InstallationMethod::AppImage,
                        scope: InstallationScope::User,
                        executable_path: Some(path.clone()),
                        package_identifier: None,
                        package_manager: None,
                        architecture: CpuArch::Unknown,
                        installation_root: Some(
                            path.parent().unwrap_or(Path::new("/")).to_path_buf(),
                        ),
                        provenance: Some("filesystem scan".into()),
                        health: HealthStatus::Unknown,
                        confidence: DetectionConfidence::Medium,
                        last_verified_at: None,
                    });
                    break;
                }
            }
        }
    }
}

fn discover_path(apps: &[Application], out: &mut Vec<Installation>) {
    for app in apps {
        for exe in &app.executable_names {
            if let Some(path) = which(exe) {
                // avoid duplicates: if already discovered with same executable path skip? keep simple
                out.push(Installation {
                    id: format!("path:{}", exe),
                    app_id: app.id.clone(),
                    installed_version: None,
                    method: InstallationMethod::Unknown,
                    scope: InstallationScope::Unknown,
                    executable_path: Some(path),
                    package_identifier: None,
                    package_manager: None,
                    architecture: CpuArch::Unknown,
                    installation_root: None,
                    provenance: Some("PATH".into()),
                    health: HealthStatus::Unknown,
                    confidence: DetectionConfidence::Low,
                    last_verified_at: None,
                });
                break;
            }
        }
    }
}

fn which(bin: &str) -> Option<PathBuf> {
    Command::new("which")
        .arg(bin)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| PathBuf::from(String::from_utf8_lossy(&o.stdout).trim()))
}

fn discover_desktop_files(apps: &[Application], out: &mut Vec<Installation>) {
    let mut dirs = Vec::new();
    if let Some(home) = std::env::var_os("HOME") {
        dirs.push(PathBuf::from(home).join(".local/share/applications"));
    }
    dirs.push(PathBuf::from("/usr/share/applications"));
    dirs.push(PathBuf::from("/usr/local/share/applications"));
    for dir in dirs {
        let entries = match fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("desktop") {
                continue;
            }
            let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            for app in apps {
                if app.desktop_ids.iter().any(|d| d == name) {
                    // parse Exec line maybe
                    let content = fs::read_to_string(&path).unwrap_or_default();
                    let exec = content
                        .lines()
                        .find_map(|l| l.strip_prefix("Exec=").map(|v| v.to_string()));
                    let exec_path = exec.and_then(|e| {
                        let first = e.split_whitespace().next()?;
                        which(first).or_else(|| Some(PathBuf::from(first)))
                    });
                    out.push(Installation {
                        id: format!("desktop:{}", name),
                        app_id: app.id.clone(),
                        installed_version: None,
                        method: InstallationMethod::Unknown,
                        scope: if path.to_string_lossy().contains("/.local/") {
                            InstallationScope::User
                        } else {
                            InstallationScope::System
                        },
                        executable_path: exec_path,
                        package_identifier: Some(name.to_string()),
                        package_manager: None,
                        architecture: CpuArch::Unknown,
                        installation_root: None,
                        provenance: Some("desktop file".into()),
                        health: HealthStatus::Unknown,
                        confidence: DetectionConfidence::Medium,
                        last_verified_at: None,
                    });
                    break;
                }
            }
        }
    }
}
