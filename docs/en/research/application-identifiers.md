# Application Identifiers Research

## Flatpak IDs
- Official: `ai.storyteller.<app>` (lowercase), e.g.
  `ai.storyteller.photocraft`, `ai.storyteller.artcraft-launcher`.
- Community: `io.tangencylabs.solidcraft`, `io.github.billsecond.mailcraft`.

## Desktop entry IDs
- `<flatpak-id>.desktop` (e.g. `ai.storyteller.pdfcraft.desktop`).

## Executable names
- Unverified; assumed lowercased app id and CamelCase variant. Treat as low-confidence signals.

## Package identifiers
- Same as Flatpak ID for repository packages (assumed; verify per distro).

## Catalog source
- Bundled TOML baseline; extensible via config for community apps.
