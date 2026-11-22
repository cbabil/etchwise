# Security policy

Etchwise erases drives, asks for administrator rights to write them, and
handles passwords, Wi-Fi keys and SSH keys. Security reports are welcome and
taken seriously.

## Supported versions

Only the [latest release](https://github.com/cbabil/etchwise/releases/latest)
gets security fixes.

## Reporting a vulnerability

Report it privately with
[Report a vulnerability](https://github.com/cbabil/etchwise/security/advisories/new)
(Security tab > Advisories). Do not open a public issue.

Please include:

- the Etchwise version and your operating system;
- what an attacker can do, and what they need for it (a crafted image list,
  a crafted image, a USB device, local access);
- steps or a file that reproduces it.

You should get a first answer within a week. Once a fix is released, the
advisory is published and credits you unless you prefer otherwise.

## What counts

- Writing to a drive the user did not choose, or to a system drive without
  the explicit override.
- An image list, image, link (`etchwise://`) or USB device that makes Etchwise
  run code, read or write files outside its own folders, or skip the SHA-256
  check.
- Passwords, Wi-Fi keys or private keys ending up anywhere other than the
  drive being written and the app's own settings.
- Anything sent over the network other than downloads of image lists and
  images.

## Good to know

- The macOS download is not notarized by Apple. Check it against the
  release's `SHA256SUMS` file.
- Images from the built-in lists are verified against the SHA-256 in the
  list. A custom image has no known hash, so only the write is verified.
- An extra repository you add is trusted as much as the built-in lists: only
  add lists from sources you trust.
