# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Added
- Initial Rust workspace implementation of artcraft-tui.
- Platform detection (Linux/macOS/Windows).
- Bundled catalog of 20 ArtCraft applications.
- Installation discovery (dpkg, flatpak, AppImage, PATH, desktop files).
- Diagnostics engine.
- Desired-state reconciliation planner.
- Backup and migration scaffolds.
- SQLite event store (`event_domains`, `pending_work`) at `$XDG_DATA_HOME`.
- JTD schemas for all domain events.
- Interactive setup wizard for preferred installation method.
- Smart `install` command (app id or `.deb`).
- GitHub release download + SHA-256 verification + install.
- TUI installer with selection, details modal, confirmation modal, live install log.
- mdBook documentation under `docs/`.
- Makefile.

### Changed
- (none yet)
EOF
echo done