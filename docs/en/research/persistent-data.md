# Persistent Data Research

## Launcher storage
- Linux: `~/.local/share/artcraft-launcher`
- macOS: `~/Library/Application Support/ArtCraft Launcher`
- Windows: `%LOCALAPPDATA%\ArtCraft Launcher`
- Log: `logs/artcraft-launcher.log` (+ `.1.log`, `.2.log`).

## App install locations (Linux)
- System packages: `/usr/bin`, `/usr/share/applications`.
- Flatpak: managed by Flatpak (system/user).
- AppImage: `~/Applications`.
- Tarball: user-chosen directory.
- Launcher sandbox: `~/.var/app/ai.storyteller.artcraft-launcher/data/artcraft-launcher/apps/<app>`
  with settings in `home/` beside it.

## App data locations
- Unverified per app. Do not assume XDG. Need investigation of each app's config/state dirs.
- Flatpak sandbox data under `~/.var/app/<id>/`.
- Launcher sandbox apps keep config inside the sandbox (`home/`).

## Migration implications
- Flatpak -> DEB may require path translation; never blindly copy whole sandbox.
- Unknown layouts require explicit user review.
