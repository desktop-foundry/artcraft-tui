# CLI Reference

## Global flags

- `--json` — emit machine-readable JSON.

## Commands

- `detect` — detect OS, distribution, architecture, package managers.
- `catalog` — list the bundled application catalog.
- `discover` — discover installed applications.
- `diagnose` — run read-only diagnostics.
- `plan` — build and display a reconciliation plan (dry-run).
- `apply` — apply a plan (with confirmation).
- `setup [--method <m>]` — first-time setup wizard.
- `install <target>` — smart install: app id or `.deb` path.
- `install-app <app-id>` — download and install an app.
- `install-deb <file.deb>` — install a local `.deb`.
- `tui` (default) — launch the TUI.
