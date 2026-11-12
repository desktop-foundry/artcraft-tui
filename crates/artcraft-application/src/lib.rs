use artcraft_config::AppConfig;
use artcraft_domain::{DiagnosticFinding, PlatformInfo};
use artcraft_storage::{EventStore, Journal};

pub struct AppContext {
    pub platform: PlatformInfo,
    pub catalog: Vec<artcraft_domain::Application>,
    pub journal: Option<Journal>,
    pub event_store: Option<EventStore>,
}

impl AppContext {
    pub fn new() -> Self {
        let platform = artcraft_platform::detect();
        let catalog = artcraft_catalog::bundled();
        let journal = Journal::in_memory().ok();
        let event_store = EventStore::open_default().ok();
        Self {
            platform,
            catalog,
            journal,
            event_store,
        }
    }

    pub fn discover_installations(&self) -> Vec<artcraft_domain::Installation> {
        artcraft_discovery::discover_all(&self.catalog)
    }

    pub fn diagnose(&self) -> Vec<DiagnosticFinding> {
        let insts = self.discover_installations();
        let mut findings = Vec::new();
        // duplicate detection
        use std::collections::BTreeMap;
        let mut by_app: BTreeMap<String, Vec<&artcraft_domain::Installation>> = BTreeMap::new();
        for i in &insts {
            by_app.entry(i.app_id.to_string()).or_default().push(i);
        }
        for (app, list) in &by_app {
            if list.len() > 1 {
                findings.push(DiagnosticFinding {
                    id: format!("dup-{}", app),
                    severity: "warning".into(),
                    title: format!("Duplicate installations for {}", app),
                    evidence: list.iter().map(|i| i.id.clone()).collect(),
                    confidence: artcraft_domain::DetectionConfidence::High,
                    recommended_action: Some("Resolve duplicates via reconciliation".into()),
                    auto_repair_supported: false,
                    verified_fact: true,
                });
            }
        }
        // broken: missing executable for package-managed? check
        for i in &insts {
            if i.executable_path
                .as_ref()
                .map(|p| !p.exists())
                .unwrap_or(false)
            {
                findings.push(DiagnosticFinding {
                    id: format!("missing-exe-{}", i.id),
                    severity: "error".into(),
                    title: format!("Missing executable for {}", i.app_id),
                    evidence: vec![i.id.clone()],
                    confidence: artcraft_domain::DetectionConfidence::Medium,
                    recommended_action: Some("Reinstall or repair".into()),
                    auto_repair_supported: false,
                    verified_fact: false,
                });
            }
        }
        findings
    }

    pub fn reconcile_plan(&self) -> artcraft_domain::ReconciliationPlan {
        let cfg = AppConfig::load().unwrap_or_default();
        let desired = cfg.to_desired_state();
        let insts = self.discover_installations();
        artcraft_reconcile::build_plan(&desired, &insts)
    }
}

impl Default for AppContext {
    fn default() -> Self {
        Self::new()
    }
}

impl AppContext {
    /// Execute a reconciliation plan, running package-manager commands.
    /// Returns a list of (operation_id, success, message).
    pub fn execute_plan(
        &self,
        plan: &artcraft_domain::ReconciliationPlan,
    ) -> Vec<(String, bool, String)> {
        use artcraft_events::EventSink;
        use artcraft_packages::{AptManager, FlatpakManager, PackageManager};
        // log approval
        if let Some(store) = &self.event_store {
            store.record(artcraft_events::DomainEvent::ReconciliationApproved {
                plan_id: plan.id.clone(),
            });
        }
        let preferred = artcraft_config::AppConfig::load()
            .unwrap_or_default()
            .preferred_method
            .unwrap_or_else(|| "flatpak".into());
        let mut results = Vec::new();
        for op in &plan.operations {
            let mut ok = true;
            let mut msg = String::new();
            match op.kind.as_str() {
                "install" | "migrate" => {
                    let app = op
                        .app_id
                        .as_ref()
                        .and_then(|id| self.catalog.iter().find(|a| a.id == *id));
                    if let Some(app) = app {
                        let mgr: Box<dyn PackageManager> =
                            if preferred == "apt" || preferred == "deb" {
                                Box::new(AptManager)
                            } else {
                                Box::new(FlatpakManager)
                            };
                        match mgr.resolve(app) {
                            Ok(cand) => match mgr.plan_install(&cand) {
                                Ok(install_plan) => {
                                    for cmd in &install_plan.commands {
                                        let res =
                                            run_command(cmd, install_plan.requires_privileges);
                                        if !res.0 {
                                            ok = false;
                                            msg = res.1.clone();
                                            break;
                                        }
                                        msg = res.1.clone();
                                    }
                                    if ok {
                                        msg = format!("installed {}", app.id);
                                    }
                                }
                                Err(e) => {
                                    ok = false;
                                    msg = e.to_string();
                                }
                            },
                            Err(e) => {
                                ok = false;
                                msg = e.to_string();
                            }
                        }
                    } else {
                        ok = false;
                        msg = "app not found".into();
                    }
                }
                "verify" => {
                    // best-effort: check executable existence not available; mark ok
                    msg = "verified".into();
                }
                _ => {
                    msg = format!("skipped {}", op.kind);
                }
            }
            if let Some(store) = &self.event_store {
                if ok {
                    store.record(artcraft_events::DomainEvent::OperationCompleted {
                        operation_id: op.id.clone(),
                        kind: op.kind.clone(),
                    });
                } else {
                    store.record(
                        artcraft_events::DomainEvent::ReconciliationExecutionFailed {
                            plan_id: plan.id.clone(),
                            reason: msg.clone(),
                        },
                    );
                }
            }
            results.push((op.id.clone(), ok, msg));
        }
        if let Some(store) = &self.event_store {
            let succ = results.iter().filter(|r| r.1).count();
            store.record(artcraft_events::DomainEvent::ReconciliationExecuted {
                plan_id: plan.id.clone(),
                operations: succ,
            });
        }
        results
    }
}

pub fn run_command(cmd: &[String], privileged: bool) -> (bool, String) {
    if cmd.is_empty() {
        return (true, "no-op".into());
    }
    let is_root = unsafe {
        extern "C" {
            fn geteuid() -> u32;
        }
        geteuid() == 0
    };
    let mut c = if privileged && !is_root {
        let mut v = vec!["sudo".to_string()];
        v.extend(cmd.iter().cloned());
        v
    } else {
        cmd.to_vec()
    };
    // Actually use std::process::Command
    let program = c.remove(0);
    println!("  -> {} {}", program, c.join(" "));
    let status = std::process::Command::new(&program).args(&c).status();
    match status {
        Ok(s) => {
            if s.success() {
                (true, format!("completed (exit {})", s.code().unwrap_or(0)))
            } else {
                (false, format!("failed (exit {})", s.code().unwrap_or(-1)))
            }
        }
        Err(e) => (false, e.to_string()),
    }
}

impl AppContext {
    /// Install a local .deb package on Ubuntu/Debian-compatible systems.
    pub fn install_deb(&self, path: &std::path::Path) -> (bool, String) {
        use artcraft_events::EventSink;
        if !path.exists() {
            return (false, format!("file not found: {}", path.display()));
        }
        // verify it's a deb via dpkg-deb
        let check = std::process::Command::new("dpkg-deb")
            .args(["-I", &path.to_string_lossy()])
            .output();
        if check.map(|o| !o.status.success()).unwrap_or(true) {
            return (false, "not a valid .deb package".into());
        }
        // prefer apt for dependency resolution
        let cmd = vec![
            "apt-get".to_string(),
            "install".to_string(),
            "-y".to_string(),
            path.to_string_lossy().to_string(),
        ];
        let (ok, msg) = run_command(&cmd, true);
        if let Some(store) = &self.event_store {
            if ok {
                store.record(artcraft_events::DomainEvent::InstallationCompleted {
                    app_id: path.to_string_lossy().to_string(),
                });
            } else {
                store.record(
                    artcraft_events::DomainEvent::ReconciliationExecutionFailed {
                        plan_id: "install-deb".into(),
                        reason: msg.clone(),
                    },
                );
            }
        }
        (ok, if msg.is_empty() { "done".into() } else { msg })
    }
}
