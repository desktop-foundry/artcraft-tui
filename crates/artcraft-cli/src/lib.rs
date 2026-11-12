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
    Apply {
        #[arg(long)]
        install: Option<String>,
        #[arg(long)]
        remove: Option<String>,
        #[arg(long)]
        yes: bool,
    },
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
        Some(Commands::Apply {
            install,
            remove,
            yes,
        }) => do_apply(&ctx, install, remove, yes),
        Some(Commands::Setup { method }) => do_setup(&ctx, method),
    }
}

fn do_setup(ctx: &artcraft_application::AppContext, method: Option<String>) -> ExitCode {
    use artcraft_events::EventSink;
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

    if let Some(store) = &ctx.event_store {
        store.record(artcraft_events::DomainEvent::PreferredMethodSelected {
            method: chosen.clone(),
        });
    }
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

fn do_apply(
    ctx: &artcraft_application::AppContext,
    install: Option<String>,
    remove: Option<String>,
    yes: bool,
) -> ExitCode {
    use artcraft_events::EventSink;
    use std::io::{self, Write};

    let mut cfg = artcraft_config::AppConfig::load().unwrap_or_default();
    let mut desired: Vec<String> = cfg.desired_apps.clone();

    if let Some(list) = install {
        for app in list.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
            if !desired.iter().any(|d| d == app) {
                desired.push(app.to_string());
            }
        }
    }
    let remove_list: Vec<String> = remove
        .as_deref()
        .unwrap_or("")
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    for app in &remove_list {
        desired.retain(|d| d != app);
    }
    cfg.desired_apps = desired.clone();
    let _ = cfg.save();

    if let Some(store) = &ctx.event_store {
        store.record(artcraft_events::DomainEvent::AppSelectionChanged {
            apps: desired.clone(),
        });
    }

    let plan = ctx.reconcile_plan();
    println!("Reconciliation plan: {}", plan.id);
    println!("{}", artcraft_reconcile::plan_to_text(&plan));

    // Removal summary
    if !remove_list.is_empty() {
        println!("Removals requested: {}", remove_list.join(", "));
    }

    if !yes {
        print!("Execute? [y/N]: ");
        io::stdout().flush().ok();
        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Aborted.");
            return ExitCode::SUCCESS;
        }
    }

    if let Some(store) = &ctx.event_store {
        store.record(artcraft_events::DomainEvent::ReconciliationApproved {
            plan_id: plan.id.clone(),
        });
    }

    // Execute installs/migrations
    let results = ctx.execute_plan(&plan);

    // Execute removals
    let mut remove_results: Vec<(String, bool, String)> = Vec::new();
    if !remove_list.is_empty() {
        let insts = ctx.discover_installations();
        let preferred = cfg
            .preferred_method
            .clone()
            .unwrap_or_else(|| "flatpak".into());
        for app in &remove_list {
            let matching: Vec<_> = insts
                .iter()
                .filter(|i| i.app_id.to_string() == *app)
                .collect();
            if matching.is_empty() {
                remove_results.push((format!("remove:{}", app), true, "not installed".into()));
                continue;
            }
            for inst in matching {
                let mgr: Box<dyn artcraft_packages::PackageManager> =
                    if preferred == "apt" || preferred == "deb" {
                        Box::new(artcraft_packages::AptManager)
                    } else {
                        Box::new(artcraft_packages::FlatpakManager)
                    };
                match mgr.plan_remove(inst) {
                    Ok(rp) => {
                        let mut ok = true;
                        let mut msg = String::new();
                        for cmd in &rp.commands {
                            let (s, m) =
                                artcraft_application::run_command(cmd, rp.requires_privileges);
                            ok = s;
                            msg = m;
                            if !ok {
                                break;
                            }
                        }
                        if ok {
                            msg = format!("removed {}", app);
                        }
                        if let Some(store) = &ctx.event_store {
                            store.record(artcraft_events::DomainEvent::OperationCompleted {
                                operation_id: format!("remove:{}", inst.id),
                                kind: "remove".into(),
                            });
                        }
                        remove_results.push((format!("remove:{}", app), ok, msg));
                    }
                    Err(e) => {
                        remove_results.push((format!("remove:{}", app), false, e.to_string()))
                    }
                }
            }
        }
    }

    // Summary
    println!();
    println!("Execution summary:");
    for (id, ok, msg) in &results {
        println!("  [{}] {} - {}", if *ok { "OK" } else { "FAIL" }, id, msg);
    }
    for (id, ok, msg) in &remove_results {
        println!("  [{}] {} - {}", if *ok { "OK" } else { "FAIL" }, id, msg);
    }
    let total_fail =
        results.iter().filter(|r| !r.1).count() + remove_results.iter().filter(|r| !r.1).count();
    println!();
    if total_fail == 0 {
        println!("All operations completed.");
    } else {
        println!("{} operation(s) failed.", total_fail);
    }
    ExitCode::SUCCESS
}
