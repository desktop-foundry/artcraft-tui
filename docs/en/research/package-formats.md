# Package Formats Research

## Verified formats (Linux)
- Flatpak (OCI registry remote) — `flatpak remote-add --user --if-not-exists artcraft-oci https://getartcraft.com/artcraft-oci.flatpakrepo`
- `.deb` (apt) — Debian/Ubuntu/Mint/Pop!_OS.
- `.rpm` (dnf/zypper) — Fedora/RHEL/openSUSE.
- AppImage — universal, x86_64/aarch64.
- Tarball (`.tar.gz`) — plain tree, x86_64/aarch64/riscv64.
- Flatpak bundle (`.flatpak`).
- Arch Linux — repository (details unverified).

## Architectures
- x86_64 (amd64)
- aarch64 (arm64)
- riscv64 (select formats, glibc 2.39+)

## Integrity
- SHA-256 checksums published in `SHA256SUMS.txt` per release.
- Package managers verify `.deb`/`.rpm` against SHA-256 in repository index.
- Downloads streamed from GitHub Releases via HTTP 302 redirects.

## Uncertainties
- Exact repository configuration commands for non-Flatpak formats (not captured from static page).
- GPG key handling for apt/dnf/zypper repos.
- Arch package repository URL and trust model.
EOF
echo done