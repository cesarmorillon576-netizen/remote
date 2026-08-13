# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this project is

Remote is an open-source, local-first device continuity platform (think independent KDE Connect alternative): phones and computers discover each other on the LAN, pair securely, and expose capabilities to each other bidirectionally (clipboard, file transfer, remote input, media control, audio/video/camera/microphone streaming). Targets: Linux (Wayland first, desktop-environment independent), Windows, Android. CLI-first; any GUI is a thin client over the same core.

**This is a docs-first project.** The design lives in `docs/` and is far ahead of the code, which is minimal exploratory scaffolding. Read the docs before writing code:

- `docs/000-vision.md` — goals, non-goals, philosophy
- `docs/001-requirements.md` — FR/NFR/SEC/PLAT requirement catalog
- `docs/002-architecture.md` — layering, crate responsibilities, dependency rules
- `docs/003-protocol.md` — Remote Protocol v1 wire spec (QUIC + CBOR, Ed25519 identity). Treated as the source of truth for wire behavior, but currently **under revision**: see `docs/design/2026-08-08-docs-revision-design.md` for the approved redesign (mTLS + certificate pinning + SAS pairing replacing app-level AUTH; Bluetooth deferred to v2; Sharing service added; Commands deferred; complete per-service message registries; P0 = discovery/pairing/session/clipboard/files).

## Commands

```sh
cargo build                 # build workspace
cargo test                  # run all tests
cargo test -p remote-core <test_name>   # run a single test in a crate
cargo run -p remote-cli     # run the CLI (binary: remote)
cargo run -p remote         # run the daemon (binary: remoted)
cargo fmt && cargo clippy   # format + lint before considering work done
```

## Workspace structure and known inconsistencies

Cargo workspace with three crates (per `002-architecture.md`):

- `remote-core` — shared library: protocol, discovery, pairing, services, transport abstraction. Must never depend on UI.
- `remote` — the daemon/long-running application (binary `remoted`, Unix daemon convention — it cannot share the `remote` binary name with the CLI because both would collide in the workspace's shared `target/`).
- `remote-cli` — CLI (binary `remote`), thin client over `remote-core`; must not duplicate core logic.

Code conventions:

- Identifiers in English (owner's decision; comments may be in Spanish). Docs in English.
- The early KDE Connect-style JSON exploration (`packet.rs`) was deleted: the Remote protocol is CBOR with integer keys and independent of KDE Connect. Implement against `003-protocol.md` (as revised) — do not reintroduce JSON packet models.
- `remote-core` starts empty on purpose; modules grow as the owner implements them following `002-architecture.md` §30's proposed layout.

## Architecture rules that must hold

From `002-architecture.md` (dependency direction: UI → application → core):

- Services talk to the protocol layer, never directly to sockets; transports are abstracted so services don't know Wi-Fi from anything else.
- Control plane (handshake, capabilities, permissions, service control — control stream) is separate from data plane (files, audio, video — dedicated QUIC streams, correlated by stream ID).
- Platform-specific code (Wayland/X11/Windows/Android) stays behind adapter interfaces; nothing platform-specific leaks into protocol or core types.
- Pairing (trust) and permissions (what a trusted device may do) are independent; privileged operations always check permission, and defaults are DENIED.
- Service failures must not tear down the session or unrelated services.

## Working agreements

- **The owner is implementing this project himself to learn Rust.** Act as mentor and reviewer: explain concepts, review code on request, answer design questions. Do not write implementation code unless he explicitly delegates a specific piece.
- The owner handles git themselves — do not commit, push, or otherwise touch git state unless explicitly asked.
- Docs follow a numbered-file convention (`docs/NNN-topic.md`), English, numbered headings, ASCII diagrams. Match that style when editing them. Design specs and implementation plans go in `docs/design/`.
