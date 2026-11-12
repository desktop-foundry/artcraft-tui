# Upstream Ecosystem Research

## Sources
- https://getartcraft.com/apps (official catalog)
- https://github.com/storytold/craft-launcher (official launcher)
- https://getartcraft.com/linux (official Linux repo docs)

## Verified facts

- Official catalog advertises **17** applications:
  PhotoCraft, VectorCraft, FilmCraft, LightCraft, PdfCraft, EffectCraft, AniCraft,
  DesignCraft, FigCraft, TextureCraft, MaterialCraft, WordCraft, SoundCraft, DeckCraft,
  CADCraft, GridCraft, Craft Anything.
- The launcher additionally manages itself (ArtCraft Launcher) plus community apps:
  - SolidCraft (Tangency Labs) — `io.tangencylabs.solidcraft`
  - MailCraft (UCS Care Group) — `io.github.billsecond.mailcraft`
- All apps are open source, written in Rust, native (macOS/Windows/Linux; many also WebAssembly).
- Flatpak application IDs follow `ai.storyteller.<app>` for official apps
  (e.g. `ai.storyteller.pdfcraft`, `ai.storyteller.artcraft-launcher`).
- Community Flatpak IDs: `io.tangencylabs.solidcraft`, `io.github.billsecond.mailcraft`.
- Flatpak OCI remote: `https://getartcraft.com/artcraft-oci.flatpakrepo`
  (add with `flatpak remote-add --user --if-not-exists artcraft-oci ...`).
- Single-app Flatpak install via `.flatpakref`:
  `https://getartcraft.com/apps/pdfcraft/pdfcraft-oci.flatpakref`.
- Linux install locations (launcher):
  - `/usr/bin`, `/usr/share/applications` for `.deb`/`.rpm`.
  - Flatpak user/system installs via Flatpak.
  - AppImage in `~/Applications`.
  - Portable tarball (plain tree).
  - Launcher-sandbox installs: `~/.var/app/ai.storyteller.artcraft-launcher/data/artcraft-launcher/apps/<app>`.
- Launcher data folder: `~/.local/share/artcraft-launcher` (Linux),
  `~/Library/Application Support/ArtCraft Launcher` (macOS),
  `%LOCALAPPDATA%\ArtCraft Launcher` (Windows).
- Launcher log: `logs/artcraft-launcher.log` in data folder, keeps previous two runs
  (`artcraft-launcher.1.log`, `artcraft-launcher.2.log`).
- Release assets naming:
  `artcraft-launcher-<ver>-linux-{x86_64,aarch64}.{AppImage,deb,rpm,flatpak,tar.gz}`
  plus `riscv64` variants and `.AppImage.zsync`.
- Every release ships `SHA256SUMS.txt`.
- Supported Linux CPU: x86_64, aarch64 (riscv64 for select formats).
- Launcher requires glibc 2.35+ (Ubuntu 22.04+, Debian 12+, Fedora 36+).
- Launcher license: MIT OR Apache-2.0.
- Apps each have their own installers on their pages; launcher is recommended.

## Uncertainties / not yet verified
- Exact `.deb`/`.rpm`/Arch repository URLs and signing keys (the getartcraft.com/linux page renders those tabs client-side; static HTML only exposed the Flatpak tab). Do not invent; require live browser capture or upstream docs.
- Per-app executable binary names (assumed lowercased app id; launcher installs to /usr/bin but exact names unverified).
- Per-app persistent data directories (XDG assumptions; actual layouts unverified, especially Flatpak sandbox paths).
- Whether every official app has a Flatpak build / package in the OCI remote (assumed yes).
- Community apps' catalog metadata endpoints.

## Differences between website and launcher catalog
- Website: 17 official apps.
- Launcher: 17 official + itself + 2 community apps (SolidCraft, MailCraft).
- Therefore product catalog must be extensible and not hardcode "17".

## Upstream changes that could break integration
- Flatpak app ID renames.
- Release asset naming changes.
- Removal of a format for an app.
- New community apps.
- Launcher sandbox storage layout changes.
