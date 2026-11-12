# Getting Started

## Build

```sh
cargo build --release
```

The binary is `target/release/artcraft`.

## First run

```sh
artcraft
```

On first launch, a wizard asks for your preferred installation method
(APT/DEB, Flatpak, AppImage, or portable tarball). The choice is stored in
`$XDG_CONFIG_HOME/artcraft-tui/config.toml`.

## Basic commands

```sh
artcraft detect          # platform and capabilities
artcraft catalog         # list known applications
artcraft discover        # find installations
artcraft diagnose        # run diagnostics
artcraft plan            # show reconciliation plan
artcraft install <app>   # download + install an app
artcraft install <file.deb>  # install a local .deb
```

Add `--json` to any command for machine-readable output.
