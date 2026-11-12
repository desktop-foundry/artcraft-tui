# Launcher Interoperability Research

## Role
- Official graphical launcher manages install/update/open of all apps.
- artcraft-tui must not duplicate blindly; distinctive capabilities: discovery, reconciliation, migration, data preservation.

## Interop points
- Detect launcher-managed installations (sandbox path).
- Detect apps installed by launcher via native packages (same as manual).
- Read launcher data folder for logs/state (read-only by default).
- Do not corrupt launcher's sandbox.

## Special storage
- Launcher Flatpak installs apps on host via `flatpak-spawn --host` when host Flatpak exists.
- Otherwise falls back to tarball unpacked inside sandbox.

## Upstream changes that could break
- Sandbox path layout.
- Self-update mechanism.
- Release asset naming.
EOF
echo done