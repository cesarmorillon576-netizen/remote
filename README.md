# Remote

Local-first device continuity between your devices: phones and computers discover each other on the LAN, pair securely, and share capabilities — clipboard sync, file transfer, remote input, media control, and more. An independent, open-source alternative in the spirit of KDE Connect.

- **Local-first.** Everything happens on your network. No accounts, no cloud, no telemetry.
- **Secure by design.** Ed25519 identities, mutual TLS with certificate pinning, user-confirmed pairing codes. Permissions are independent from pairing and default to denied.
- **CLI-first.** The daemon (`remoted`) and the CLI (`remote`) are the product; any GUI is a thin client over the same core.
- **Cross-platform targets:** Linux (Wayland first, desktop-environment independent), Windows, Android.

## Status

Early design stage. The protocol and architecture are specified in `docs/` — the specification is far ahead of the code, which is just workspace scaffolding for now. Expect everything to change.

## Documentation

The design lives in `docs/` and is the source of truth:

| Document | Contents |
|---|---|
| [`docs/000-vision.md`](docs/000-vision.md) | Goals, non-goals, philosophy |
| [`docs/001-requirements.md`](docs/001-requirements.md) | Functional and non-functional requirements |
| [`docs/002-architecture.md`](docs/002-architecture.md) | Layering, crates, dependency rules |
| [`docs/003-protocol.md`](docs/003-protocol.md) | Remote Protocol v1 wire spec (QUIC + CBOR) |

Design specs and implementation plans go in `docs/design/`.

## Building

Rust workspace with three crates: `remote-core` (shared library), `remote` (daemon, binary `remoted`), and `remote-cli` (CLI, binary `remote`).

```sh
cargo build       # build everything
cargo test        # run tests
cargo run -p remote-cli   # run the CLI
cargo run -p remote       # run the daemon
```
