# Etchwise

Etchwise writes operating system images to SD cards, USB drives and SSDs, for
Raspberry Pi computers and for PCs such as the Intel NUC, and sets the system
up for its first boot (user, password, SSH keys, host name, Wi-Fi, time zone,
keyboard).

It is a Rust + Tauri + React port of
[Raspberry Pi Imager](https://github.com/raspberrypi/rpi-imager) v2.0.7, with
its own name and icon, a second list of images for PCs, and no usage
statistics.

**Platforms.** Rust and Tauri run on macOS, Linux and Windows, and most of
Etchwise is portable (image lists, download, decompress, verify, first-boot
setup, the UI). The part that touches the drives is per operating system:
listing them, unmounting and ejecting, and opening the raw device with the
admin prompt. That layer (`crates/imager-platform`) exists for **macOS only**
today, so Etchwise lists drives and writes them on macOS; on Linux and
Windows it builds, but lists no drives and refuses to write.

## What it can write

- **The official Raspberry Pi list**, as in Raspberry Pi Imager (Raspberry Pi
  tab of the Device step).
- **Etchwise's own list** (Other tab), built into the app
  ([`repos/etchwise.json`](repos/etchwise.json)), for the Intel NUC and other
  UEFI PCs:
  - **Kura**, the home-server OS ([cbabil/kura](https://github.com/cbabil/kura)),
    read from its latest release: "Kura &lt;version&gt;" for the Raspberry Pi 5,
    "Kura &lt;version&gt; installer (PC)" (a small USB key that installs Kura on
    the PC's internal drive over wired internet) and "Kura &lt;version&gt; (PC)"
    (runs from the drive it is written to).
  - Ubuntu Server 26.04.1 LTS, Ubuntu Desktop 26.04.1 LTS, Debian 13.7
    netinst and Proxmox VE 9.2 installers.
- **Your own image** (Use custom), and **an extra list** of your own (App
  Options > Extra Repository), merged on top.
- **Erase** (FAT32 format).

Each image is downloaded (or taken from the cache), written, read back and
checked against its SHA-256.

## First-boot setup

The setup screens (host name, locale, user, Wi-Fi, remote access) are turned
into what each system reads at its first boot:

| Image | What Etchwise writes |
|---|---|
| Raspberry Pi OS and other Pi images | cloud-init files or `firstrun.sh`, as Raspberry Pi Imager |
| Ubuntu Server (PC) | Ubuntu autoinstall `user-data` on a small `CIDATA` partition added after the image |
| Kura (Pi 5 and PC) | `kura-setup.json` on the boot / EFI partition |

The user name must be lowercase letters, digits, `_` or `-`, and not a name
the system already uses (for example `root`): Kura ignores a setup file whose
user name is taken. SSH keys can be typed, pasted (PuTTY keys are converted)
or loaded from a file.

## Build and run

Requirements: macOS, [rustup](https://rustup.rs) (the toolchain is pinned in
`rust-toolchain.toml`), [bun](https://bun.sh).

```sh
make install   # JS dependencies and git hooks
make dev       # run with hot reload (REPO=file.json for another OS list)
make build     # release .app
make dmg       # release .app and .dmg
make check     # formatting, lint, tests and the 90% coverage gates
make clean     # remove build output
```

There is no CI: run `make check` before pushing.

## Layout

| Path | What it is |
|---|---|
| `crates/imager-core` | OS lists, settings, first-boot setup generators, write pipeline (download, decompress, write, verify, customise), cache, erase, Secure Boot signing |
| `crates/imager-platform` | macOS: drive list and unmount/eject (DiskArbitration), raw device access (authopen), Wi-Fi name, keychain, location |
| `crates/imager-rpiboot` | USB boot of Compute Modules (rpiboot, fastboot); off by default (App Options > Debug) |
| `apps/desktop/src-tauri` | The Tauri app: one `imager` command and events, CLI mode, `etchwise://` links |
| `apps/desktop/src` | The UI (React + MUI), with the upstream translations (27 languages) |
| `repos/etchwise.json` | Etchwise's own image list |

## Known limits

- Writes drives on macOS only (see Platforms): Linux (udisks2 or sysfs plus
  polkit) and Windows (volume locks, raw disk handles) each need their own
  `imager-platform` layer.
- Raspberry Pi Connect's sign-in returns its token to `rpi-imager://`; paste
  the token by hand.
- Compute Module USB boot is tested against mocked transports only.

## Licence

Apache License 2.0 ([LICENSE](LICENSE)). Etchwise is based on Raspberry Pi
Imager, Copyright (C) 2020 Raspberry Pi Ltd; see [NOTICE](NOTICE). "Raspberry
Pi" is a trademark of Raspberry Pi Ltd; Etchwise is not affiliated with or
endorsed by Raspberry Pi Ltd.
