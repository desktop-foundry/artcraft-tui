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
    Setup,
    Tui,
}

pub fn run() -> ExitCode {
    let cli = Cli::parse();
    let ctx = artcraft_application::AppContext::new();
    match cli.command {
        None | Some(Commands::Tui) => {
            // launch tui
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
        Some(Commands::Setup) => {
            // first-launch wizard minimal
            println!("Detected platform: {:?}", ctx.platform.os);
            println!("Supported methods:");
            println!("  apt/flatpak/appimage/tarball");
            println!("Run with config to set preference. Saving default: flatpak");
            let mut cfg = artcraft_config::AppConfig::load().unwrap_or_default();
            if cfg.preferred_method.is_none() {
                cfg.preferred_method = Some("flatpak".into());
                let _ = cfg.save();
            }
            println!("Saved preference: {:?}", cfg.preferred_method);
            let findings = ctx.diagnose();
            println!("Initial diagnosis findings: {}", findings.len());
            ExitCode::SUCCESS
        }
    }
}
