# Contributing to Etchwise

Thank you for helping. Bug reports, fixes, new images for the list and tests
on real hardware are all welcome, and so are small changes.

## Reporting

- **Bugs and ideas:** open an [issue](https://github.com/cbabil/etchwise/issues/new/choose).
- **Security problems:** report them privately, see [SECURITY.md](SECURITY.md).

Linux and Windows have never written a real drive (see the README's Platform
support). A report of a build and a write on either, working or not, is one
of the most useful things you can send.

## Setting up

You need macOS, [rustup](https://rustup.rs) and [bun](https://bun.sh).

```sh
make install   # JS dependencies and git hooks
make dev       # run the app with hot reload
make check     # formatting, lint, tests and the 90% coverage gates
```

`make help` lists every target. The README's Project layout says what each
folder holds.

## Making a change

1. Fork the repository and branch from `dev`.
2. Write the test first, then the change. Coverage must stay at 90% or above.
3. Run `make check`. If you touched `crates/imager-platform`, also run
   `make cross-check` (needs `zig` and `cargo-zigbuild`).
4. Write commit messages as [Conventional Commits](https://www.conventionalcommits.org),
   for example `fix(storage): list drives without a partition table`.
5. Open a pull request against `dev` and fill in the template.

There is no CI: the pre-commit hook and `make check` on your machine are the
checks, so please run them.

## Ground rules

- No telemetry and no network calls other than image lists and images.
- Never commit secrets, tokens or email addresses; the pre-commit hook blocks
  them.
- Anything that writes to a drive needs a test, and a note in the pull
  request saying whether a real drive was written.
- The UI keeps Raspberry Pi Imager's look and its translations. New text
  needs to work in all 27 languages or be marked English only.

## Adding an image to the list

Edit [`repos/etchwise.json`](repos/etchwise.json): the download `url`, both
sizes and both SHA-256 values from the vendor's own checksum file, and the
`devices` it runs on. Use a versioned URL, not a "latest" one, or the hash
breaks at the next release. `make test` checks that the list still loads.

## Licence

By contributing you agree that your work is released under the
[Apache License 2.0](LICENSE).
