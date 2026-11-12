use clap::{Parser, Subcommand};
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "artcraft", version, about = "ArtCraft installation reconciler")]
pub struct Cli {
    #[arg(long, global = true)]
    pub json: bool,
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    Detect,
    Catalog,
    Discover,
    Diagnose,
    Plan,
    Setup {
        #[arg(long)]
        method: Option<String>,
    },
    Tui,
}

pub fn run() -> ExitCode {
    let cli = Cli::parse();
    let ctx = artcraft_application::AppContext::new();
    // First-launch wizard.
    if matches!(cli.command, None | Some(Commands::Tui)) {
        let cfg = artcraft_config::AppConfig::load().unwrap_or_default();
        if cfg.preferred_method.is_none() {
            let _ = do_setup(&ctx, None);
        }
    }
    match cli.command {
        None | Some(Commands::Tui) => {
            if let Err(e) = artcraft_tui::run(ctx) {
                eprintln!("tui error: {}", e);
                return ExitCode::FAILURE;
            }
            ExitCode::SUCCESS
        }
        Some(Commands::Detect) => {
            let p = ctx.platform;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&p).unwrap_or_default());
            } else {
                println!("OS: {:?}", p.os);
                println!("Distro: {} {}", p.distro_id, p.version);
                println!("Arch: {:?}", p.arch);
                println!("Package managers: {:?}", p.package_managers);
                println!("Flatpak: {}", p.flatpak_available);
                println!("Root: {}", p.is_root);
            }
            ExitCode::SUCCESS
        }
        Some(Commands::Catalog) => {
            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&ctx.catalog).unwrap_or_default()
                );
            } else {
                for a in &ctx.catalog {
                    println!(
                        "{} - {}{}",
                        a.id,
                        a.display_name,
                        if a.is_community { " (community)" } else { "" }
                    );
                }
            }
            ExitCode::SUCCESS
        }
        Some(Commands::Discover) => {
            let insts = ctx.discover_installations();
            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&insts).unwrap_or_default()
                );
            } else {
                if insts.is_empty() {
                    println!("No installations found.");
                }
                for i in &insts {
                    println!(
                        "{} | {} | {:?} | {:?} | {}",
                        i.id,
                        i.app_id,
                        i.method,
                        i.health,
                        i.installed_version.clone().unwrap_or_default()
                    );
                }
            }
            ExitCode::SUCCESS
        }
        Some(Commands::Diagnose) => {
            let findings = ctx.diagnose();
            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&findings).unwrap_or_default()
                );
            } else {
                if findings.is_empty() {
                    println!("No findings.");
                }
                for f in &findings {
                    println!("[{}] {} - {}", f.severity, f.title, f.evidence.join(", "));
                }
            }
            ExitCode::SUCCESS
        }
        Some(Commands::Plan) => {
            let plan = ctx.reconcile_plan();
            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&plan).unwrap_or_default()
                );
            } else {
                println!("{}", artcraft_reconcile::plan_to_text(&plan));
            }
            ExitCode::SUCCESS
        }
        Some(Commands::Setup { method }) => do_setup(&ctx, method),
    }
}

fn do_setup(ctx: &artcraft_application::AppContext, method: Option<String>) -> ExitCode {
    use std::io::{self, Write};
    println!("Detected platform: {:?}", ctx.platform.os);
    println!(
        "Distro: {} {}",
        ctx.platform.distro_id, ctx.platform.version
    );
    println!("Architecture: {:?}", ctx.platform.arch);
    println!();

    let mut methods: Vec<(&str, &str, bool)> = Vec::new();
    match ctx.platform.os {
        artcraft_domain::Platform::Linux => {
            if ctx
                .platform
                .package_managers
                .iter()
                .any(|m| m == "apt" || m == "dpkg")
            {
                methods.push(("apt", "APT repository / DEB", true));
            }
            if ctx.platform.flatpak_available {
                methods.push(("flatpak", "Flatpak", !methods.iter().any(|m| m.2)));
            }
            methods.push(("appimage", "AppImage", methods.is_empty()));
            methods.push(("tarball", "Portable tarball", false));
        }
        artcraft_domain::Platform::MacOS => {
            methods.push(("homebrew", "Homebrew Cask", true));
            methods.push(("dmg", "macOS DMG", false));
            methods.push(("tarball", "Portable tarball", false));
        }
        artcraft_domain::Platform::Windows => {
            methods.push(("msi", "Windows MSI", true));
            methods.push(("portable", "Windows portable", false));
            methods.push(("tarball", "Portable tarball", false));
        }
        _ => {
            methods.push(("tarball", "Portable tarball", true));
        }
    }

    let chosen = if let Some(m) = method {
        m
    } else {
        println!("Choose your preferred installation method:");
        println!();
        for (i, (_id, label, rec)) in methods.iter().enumerate() {
            if *rec {
                println!("  {}. {}   (Recommended)", i + 1, label);
            } else {
                println!("  {}. {}", i + 1, label);
            }
        }
        println!();
        print!("Enter number [1]: ");
        io::stdout().flush().ok();
        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();
        let idx: usize = input.trim().parse().unwrap_or(1);
        let idx = if idx == 0 || idx > methods.len() {
            1
        } else {
            idx
        };
        methods[idx - 1].0.to_string()
    };

    println!();
    println!("Selected preferred method: {}", chosen);

    let mut cfg = artcraft_config::AppConfig::load().unwrap_or_default();
    cfg.preferred_method = Some(chosen.clone());
    if let Err(e) = cfg.save() {
        eprintln!("save error: {}", e);
    }
    println!("Saved to {}", artcraft_config::AppConfig::path().display());

    let findings = ctx.diagnose();
    println!();
    println!("Initial diagnosis findings: {}", findings.len());
    for f in &findings {
        println!("  [{}] {}", f.severity, f.title);
    }
    println!();
    println!("Run `artcraft plan` to create a reconciliation plan.");
    ExitCode::SUCCESS
}
