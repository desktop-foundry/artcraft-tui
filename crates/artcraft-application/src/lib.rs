use artcraft_catalog;
use artcraft_config::AppConfig;
use artcraft_discovery;
use artcraft_domain::{DiagnosticFinding, PlatformInfo};
use artcraft_platform;
use artcraft_reconcile;
use artcraft_storage::Journal;

pub struct AppContext {
    pub platform: PlatformInfo,
    pub catalog: Vec<artcraft_domain::Application>,
    pub journal: Option<Journal>,
}

impl AppContext {
    pub fn new() -> Self {
        let platform = artcraft_platform::detect();
        let catalog = artcraft_catalog::bundled();
        let journal = Journal::in_memory().ok();
        Self {
            platform,
            catalog,
            journal,
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
            if i.executable_path.as_ref().map(|p| !p.exists()).unwrap_or(false) {
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
