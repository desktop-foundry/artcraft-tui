# Security and Integrity Research

## Requirements
- HTTPS only; validate redirects; trusted-host policy.
- SHA-256 verification of downloaded artifacts against release metadata.
- Never execute untrusted downloads; avoid shell pipelines.
- Separate unprivileged orchestration from privileged install.
- Do not run TUI as root.

## Checksums
- `SHA256SUMS.txt` per release.
- Package managers verify repository packages against index checksums.

## Threats
- TOCTOU between verify and install: use safe temp files, atomic moves.
- Path traversal in archives: validate paths.
- Symlink escape: handle carefully.

## Publisher authenticity
- Checksum match != publisher authenticity; prefer signature verification when supported (investigate upstream signing).
