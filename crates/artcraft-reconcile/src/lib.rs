use artcraft_domain::{Installation, ReconciliationOperation, ReconciliationPlan};

pub fn build_plan(
    desired: &artcraft_domain::DesiredState,
    installations: &[Installation],
) -> ReconciliationPlan {
    let mut ops = Vec::new();
    let mut counter = 0usize;
    for app_id in &desired.desired_apps {
        let insts: Vec<&Installation> = installations
            .iter()
            .filter(|i| i.app_id == *app_id)
            .collect();
        if insts.is_empty() {
            counter += 1;
            ops.push(ReconciliationOperation {
                id: format!("op-{}", counter),
                kind: "install".into(),
                app_id: Some(app_id.clone()),
                from: None,
                to: Some(desired.preferred_method.clone()),
                requires_privileges: desired.preferred_method.is_package_managed(),
                description: format!("Install {} via {:?}", app_id, desired.preferred_method),
            });
            continue;
        }
        // group by method
        let preferred_present = insts
            .iter()
            .any(|i| i.method == desired.preferred_method);
        let nonpreferred: Vec<&Installation> =
            insts.iter().filter(|i| i.method != desired.preferred_method).copied().collect();
        if insts.len() > 1 {
            counter += 1;
            ops.push(ReconciliationOperation {
                id: format!("op-{}", counter),
                kind: "resolve_duplicate".into(),
                app_id: Some(app_id.clone()),
                from: None,
                to: Some(desired.preferred_method.clone()),
                requires_privileges: true,
                description: format!(
                    "Resolve {} duplicate installations toward {:?}",
                    app_id, desired.preferred_method
                ),
            });
        }
        if !preferred_present {
            // migrate from first nonpreferred
            if let Some(src) = nonpreferred.first() {
                counter += 1;
                ops.push(ReconciliationOperation {
                    id: format!("op-{}", counter),
                    kind: "migrate".into(),
                    app_id: Some(app_id.clone()),
                    from: Some(src.method.clone()),
                    to: Some(desired.preferred_method.clone()),
                    requires_privileges: true,
                    description: format!(
                        "Migrate {} from {:?} to {:?}",
                        app_id, src.method, desired.preferred_method
                    ),
                });
            }
        }
        // verify
        counter += 1;
        ops.push(ReconciliationOperation {
            id: format!("op-{}", counter),
            kind: "verify".into(),
            app_id: Some(app_id.clone()),
            from: None,
            to: None,
            requires_privileges: false,
            description: format!("Verify {}", app_id),
        });
    }
    ReconciliationPlan {
        id: "plan-1".into(),
        operations: ops,
        dry_run: true,
    }
}

pub fn plan_to_text(plan: &ReconciliationPlan) -> String {
    let mut s = String::new();
    s.push_str(&format!("Plan {} (dry_run={})\n", plan.id, plan.dry_run));
    for op in &plan.operations {
        s.push_str(&format!("- {} {}: {}\n", op.id, op.kind, op.description));
    }
    s
}
