use artcraft_domain::{CpuArch, Platform, PlatformInfo};
use std::fs;
use std::process::Command;

pub fn detect() -> PlatformInfo {
    let mut info = PlatformInfo::default();
    #[cfg(target_os = "linux")]
    {
        detect_linux(&mut info);
    }
    #[cfg(target_os = "macos")]
    {
        info.os = Platform::MacOS;
        info.arch = arch();
    }
    #[cfg(target_os = "windows")]
    {
        info.os = Platform::Windows;
        info.arch = arch();
    }
    info.arch = arch();
    if info.os == Platform::Unknown {
        info.os = Platform::Unknown;
    }
    // xdg
    if let Some(base) = directories::BaseDirs::new() {
        info.xdg_data_home = Some(base.data_local_dir().to_path_buf());
        info.xdg_config_home = Some(base.config_dir().to_path_buf());
    }
    #[cfg(unix)]
    {
        info.is_root = libc_root();
    }
    #[cfg(not(unix))]
    {
        info.is_root = false;
    }
    info.available_disk_bytes = disk_space(&info);
    info
}

fn arch() -> CpuArch {
    match std::env::consts::ARCH {
        "x86_64" => CpuArch::X86_64,
        "aarch64" => CpuArch::Aarch64,
        "riscv64" => CpuArch::Riscv64,
        _ => CpuArch::Unknown,
    }
}

#[cfg(target_os = "linux")]
fn detect_linux(info: &mut PlatformInfo) {
    info.os = Platform::Linux;
    // os-release
    if let Ok(contents) = fs::read_to_string("/etc/os-release") {
        let mut id = String::new();
        let mut id_like = Vec::new();
        let mut version = String::new();
        for line in contents.lines() {
            if let Some(v) = line.strip_prefix("ID=") {
                id = v.trim_matches('"').to_string();
            } else if let Some(v) = line.strip_prefix("ID_LIKE=") {
                id_like = v
                    .trim_matches('"')
                    .split_whitespace()
                    .map(String::from)
                    .collect();
            } else if let Some(v) = line.strip_prefix("VERSION_ID=") {
                version = v.trim_matches('"').to_string();
            }
        }
        info.distro_id = id;
        info.distro_like = id_like;
        info.version = version;
    }
    // package managers
    let candidates = [
        ("apt", "apt"),
        ("dpkg", "dpkg"),
        ("dnf", "dnf"),
        ("rpm", "rpm"),
        ("zypper", "zypper"),
        ("pacman", "pacman"),
        ("flatpak", "flatpak"),
    ];
    for (name, bin) in candidates {
        if command_exists(bin) {
            info.package_managers.push(name.to_string());
        }
    }
    info.flatpak_available = command_exists("flatpak");
}

fn command_exists(bin: &str) -> bool {
    Command::new("which")
        .arg(bin)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn disk_space(info: &PlatformInfo) -> Option<u64> {
    // best-effort: use df on /
    let out = Command::new("df").arg("-B1").arg("/").output().ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    for line in text.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            return parts[3].parse().ok();
        }
    }
    let _ = info;
    None
}

#[cfg(unix)]
fn libc_root() -> bool {
    unsafe {
        extern "C" {
            fn geteuid() -> u32;
        }
        geteuid() == 0
    }
}
