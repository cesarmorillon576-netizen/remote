# Remote — Documentation Revision Design

**Date:** 2026-08-08
**Scope:** Revision of `docs/000-vision.md`, `docs/001-requirements.md`, `docs/002-architecture.md`, `docs/003-protocol.md`, plus the crate rename it implies.
**Status:** Approved design, pending implementation.

---

## 1. Goal

Bring the four foundational documents into a consistent, implementable state. The current protocol document claims "Stable Specification" with "no undefined wire-level behavior", but a review found critical gaps (undefined Bluetooth encryption, unbound TLS/identity relationship, one-directional authentication, missing message registries, uncorrelatable data streams) and cross-document inconsistencies (services defined in requirements/architecture but absent from the protocol, unassigned priorities, three coexisting naming conventions).

This revision fixes those issues. It is a revision, not a rewrite: all four documents keep their current structure, numbered-heading style, ASCII diagrams, and English prose.

## 2. Decisions (settled with the project owner)

1. **Bluetooth is deferred to v2.** Protocol v1 is defined over QUIC/UDP only. The transport abstraction remains in the architecture; Bluetooth becomes a short "future transports" note.
2. **Naming: everything `remote-*`.** Crates become `remote` (daemon, binary `remoted` per Unix daemon convention), `remote-cli`, `remote-core`. The CLI binary is `remote`. The daemon binary cannot also be `remote`: two bin targets with the same output filename collide in `target/` (cargo warns today, hard error in the future). The rename of directories and Cargo manifests is in scope (the workspace has ~3 source files; this is the cheap moment).
3. **P0 core = base + clipboard + files.** P0: discovery, pairing, secure session, capability/permission negotiation, clipboard sync, file transfer. P1: remote input (mouse/keyboard), media control, sharing. P2: audio, video, camera, microphone, window control, notifications, GUI. P3: commands, Bluetooth, remaining future items.
4. **Sharing enters Protocol v1; Commands moves to v2/P3.** Sharing (send text/URL) gets a service ID and messages. Commands (remote command execution) is removed from v1 protocol scope and marked P3 in requirements.

## 3. Security model change (Protocol v1)

Replace the current layered scheme (unspecified TLS certificates + application-level Ed25519 challenge-response) with the proven local-first pattern:

### 3.1 Identity

- Each installation generates one Ed25519 key pair and wraps it in a **self-signed X.509 certificate** used for QUIC/TLS 1.3.
- The certificate *is* the device identity for authentication purposes. `device_id` (UUIDv4) remains as the stable installation identifier carried in discovery and HELLO.
- There is exactly one identity key; the previous split between "pairing key" and unspecified TLS key disappears.

### 3.2 Authentication

- **Mutual TLS with certificate pinning.** Both peers present their certificates during the QUIC handshake. A session with a *paired* device is authenticated by verifying that the presented certificate exactly matches the pinned one (byte-equality or SPKI hash equality — spec picks SPKI SHA-256).
- The application-level `AUTH` / `AUTH_RESPONSE` messages, the orphan challenge, and the ambiguous signature serialization are **removed**. Authentication is mutual and channel-bound by construction.
- A connection from an unknown certificate is allowed to proceed only into the pairing flow; it cannot reach privileged services.

### 3.3 Pairing

- Pairing runs **inside** an established (encrypted but not yet trusted) TLS session.
- Both devices derive a **short authentication string (SAS)**: a 6–8 digit numeric code computed as a hash over both certificate fingerprints plus fresh nonces from both sides (exact derivation defined in the spec: SHA-256 over a domain-separation string, both SPKI hashes sorted, both nonces; truncated to decimal digits).
- The user confirms the code matches on both screens. Confirmation defeats MITM during pairing without external infrastructure. On acceptance, both sides pin the peer certificate and persist the pairing record.
- QR-code pairing remains possible later: the QR carries the certificate fingerprint, replacing the visual comparison.
- `PAIR_*` messages get message-type IDs in the core registry, integer CBOR keys, and a place in the session state machine.

### 3.4 Session establishment (revised flow)

```text
QUIC connect (TLS 1.3, both certs presented)
    ↓
HELLO ↔ HELLO_RESPONSE
    ↓
[peer certificate pinned?]
    ├── yes → continue
    └── no  → PAIR_REQUEST / PAIR_CHALLENGE / PAIR_RESPONSE / PAIR_ACCEPT|REJECT
              (SAS confirmation by user on both devices)
    ↓
CAPABILITIES ↔ CAPABILITIES_RESPONSE
    ↓
PERMISSIONS ↔ PERMISSIONS_RESPONSE   (both directions)
    ↓
SESSION_READY (sent by each side; session established when both sent and received)
```

Session states gain an explicit `PAIRING` state.

## 4. Protocol document (003) — other changes

1. **Status** changes to `Draft`; the "no undefined wire-level behavior" claim is removed until it is true.
2. **Bluetooth sections removed** (§8 and related); replaced by one short "Future Transports" section. Transport abstraction section stays.
3. **Complete message-type registry.** The envelope's `type: u16` is interpreted **within the namespace of the `service` field**. Every service gets its own table (starting at 0x0001) covering all its messages: clipboard (UPDATE, REQUEST, RESPONSE), file transfer (OFFER, ACCEPT, REJECT, COMPLETE, CANCEL), input (MOUSE_MOVE, MOUSE_BUTTON, MOUSE_SCROLL, KEY_DOWN, KEY_UP, TEXT_INPUT), window (LIST, FOCUS, MINIMIZE, MAXIMIZE, RESTORE, MOVE, RESIZE, CLOSE), media (commands + events), sharing (SHARE_TEXT, SHARE_URL), device (INFO, BATTERY, POWER), notifications (POST, DISMISS, CLEAR). Every payload gets a documented integer-key CBOR table (as HELLO already has). Typed events use their own message types with the EVENT flag; the generic `EVENT 0x0080` core message is dropped.
4. **Sharing service added**: service ID `0x0C00`, capability `0x000E SHARING`, permission `0x00B0 SHARE_RECEIVE`.
5. **Commands service**: not present in v1 (no reserved IDs beyond a note that the range is available for v2).
6. **Stream correlation.** `STREAM_OPEN` carries a `stream_id` (UUIDv4) chosen by the requester. The data-plane QUIC stream begins with exactly those 16 bytes, binding it to the negotiated service/type/options. FILE streams reference `transfer_id`/`file_id` in `STREAM_OPEN` options.
7. **Discovery wire format.** Advertisement and `DISCOVER` datagrams share a small header: magic `RMT1` (4 bytes) + packet type (u8: 0x01 ADVERTISEMENT, 0x02 DISCOVER) + CBOR payload. IPv6 discovery moves off `ff02::1` to the dedicated link-local multicast group `ff02::524d:5431`. A privacy note documents that the persistent `device_id` is broadcast in clear and names rotating identifiers as a future mitigation.
8. **Key code registry.** Keyboard `key_code` values are defined as **USB HID Usage IDs (Keyboard/Keypad page 0x07)**, referenced normatively, with a short table of Remote-relevant extras (media keys via Consumer page noted for v2).
9. **Direction semantics.** `direction` in `SERVICE_START`/`STREAM_OPEN` is defined from the perspective of the requesting peer (`SEND` = requester transmits payload data to the peer).
10. **Sensitive-data, resource-limit, error-code, rate-limiting, clipboard-loop sections** stay as-is apart from renumbering and the removal of AUTH-related items; error codes `AUTH_FAILED`, `INVALID_SIGNATURE`, `INVALID_CHALLENGE` are repurposed/kept for pairing and pinning failures (`PINNING_MISMATCH` added).

## 5. Requirements document (001)

1. **Every requirement gets a priority tag** (P0–P3) per the scheme in §2.3 of this design; the "priorities will eventually be assigned" text is replaced by the actual assignment.
2. **Commands (FR-CMD-\*)** marked P3 / future; text notes it is out of Protocol v1 scope.
3. **Sharing (FR-SHARE-\*)** marked P1 and referenced to the protocol's SHARING service.
4. **Bluetooth requirements** (FR-TRANSPORT-003 and mentions) marked future/P3.
5. **New security requirement**: pairing shall present a short verification code on both devices and require the user to confirm it (SEC-011), and authentication shall be mutual (SEC-002 text updated to mTLS + pinning language).
6. Success criteria (§31) reordered so the P0 path (install → discover → pair → capabilities → files + clipboard) reads as the first milestone.

## 6. Architecture document (002)

1. Crate names updated throughout: `remote` (daemon), `remote-cli`, `remote-core`; §5/§29 workspace listings updated; `connectgui` in §148-style diagrams becomes `remote-gui (future)`.
2. Services lists (§10 and diagrams) aligned with the protocol: Sharing included, Commands removed/marked future.
3. Transport diagrams show Wi-Fi/QUIC as the v1 transport and Bluetooth explicitly labeled "future".
4. Security sections updated to name the mTLS + pinning + SAS model (brief; the protocol doc owns the details).

## 7. Vision document (000)

Light touch only:
1. Naming consistency (`remote-*` where crates are mentioned; none currently — mostly unaffected).
2. §6 Connectivity: one line clarifying Bluetooth is a future direction, not part of the initial implementation.
3. No structural changes.

## 8. Crate rename (code)

- Workspace members become `remote` (daemon), `remote-cli`, `remote-core`, in directories `remote/`, `remote-cli/`, `remote-core/` (the repo root being named `remote` does not conflict with a member directory of the same name).
- `Cargo.toml` workspace members and each crate's `name` updated; `remote-cli` binary name set to `remote`; the daemon's binary set to `remoted` via `[[bin]]` to avoid the output-filename collision with the CLI.
- The existing `connectcore/src/{discovery,packet}.rs` move unchanged into `remote-core`.
- `cargo build` must pass after the rename.

## 9. Out of scope

- Any protocol implementation work beyond the crate rename.
- Writing new documents (ADR templates, README) — may be proposed later.
- Bluetooth design (v2), Commands service design (v2), clipboard history, adaptive-streaming renegotiation messages (noted as v2 candidates in the protocol's future-work section).

## 10. Acceptance criteria

1. The four documents are mutually consistent: every service, capability, permission, and priority referenced in one document exists coherently in the others.
2. Protocol v1 defines a wire ID and integer-key payload table for every message it names.
3. The security model contains no unbound identities, no one-directional authentication, and no undefined encryption path.
4. `cargo build` succeeds with the renamed crates.
5. Protocol status reads `Draft` with a short "path to Stable" note (implement + interop-test before flipping).
