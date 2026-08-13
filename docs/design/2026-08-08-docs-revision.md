# Documentation Revision Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Apply the approved revision in `docs/design/2026-08-08-docs-revision-design.md` to the four foundational docs and rename the workspace crates to `remote-*`.

**Architecture:** Task 1 renames the crates and makes the workspace build. Tasks 2–4 rewrite `003-protocol.md` in three passes (transport/discovery, security model, message registries). Tasks 5–7 align `001-requirements.md`, `002-architecture.md`, `000-vision.md`. Task 8 is a cross-doc consistency sweep plus CLAUDE.md update.

**Tech Stack:** Markdown docs; Rust/Cargo workspace (edition 2024) for the rename.

## Global Constraints

- **NEVER run any git command that mutates state (add/commit/push/restore/mv). The owner handles git themselves.** Use plain `mv` for renames, not `git mv`. Skip every "Commit" step convention from the executing skill.
- Docs are English, numbered `#`/`##` headings, ASCII diagrams in fenced ```text blocks. Match the existing style of each file.
- Every protocol message payload documents its CBOR map with **integer keys** (a table `ID | field | type`), like HELLO does today.
- The spec (`docs/design/2026-08-08-docs-revision-design.md`) is the authority; if this plan and the spec conflict, the spec wins and the conflict must be reported.
- Section numbers in `003-protocol.md` will shift as sections are removed/added. Renumber sequentially at the end of Task 4, not during Tasks 2–3.

---

### Task 1: Crate rename and workspace fix

**Files:**
- Rename: `connect/` → `remote/`, `connectcli/` → `remote-cli/`, `connectcore/` → `remote-core/`
- Modify: `Cargo.toml` (workspace members), `remote/Cargo.toml`, `remote-cli/Cargo.toml`, `remote-core/Cargo.toml`, `remote-core/src/discovery.rs`

**Interfaces:**
- Produces: a building workspace with crates `remote` (binary `remoted`), `remote-cli` (binary `remote`), `remote-core`. Later tasks reference these names in docs.

- [ ] **Step 1: Rename directories**

```bash
mv connect remote && mv connectcli remote-cli && mv connectcore remote-core
```

- [ ] **Step 2: Update root `Cargo.toml`** — replace the members list, adding the daemon (it is currently missing):

```toml
[workspace]
resolver = "2"

members = [
    "remote",
    "remote-cli",
    "remote-core"
]
```

- [ ] **Step 3: Update crate manifests**

`remote/Cargo.toml` — `name = "remote"`, and the daemon binary named `remoted` (Unix daemon convention; also avoids an output-filename collision with the CLI's `remote` binary in the shared `target/`):

```toml
[package]
name = "remote"
version = "0.1.0"
edition = "2024"

[[bin]]
name = "remoted"
path = "src/main.rs"

[dependencies]
```

`remote-core/Cargo.toml`: `name = "remote-core"` (dependencies unchanged).
`remote-cli/Cargo.toml`:

```toml
[package]
name = "remote-cli"
version = "0.1.0"
edition = "2024"

[[bin]]
name = "remote"
path = "src/main.rs"

[dependencies]
remote-core = { path = "../remote-core" }
```

- [ ] **Step 4: Clean out the exploration code.** (Owner's decision: English identifiers; delete the KDE-style exploration.) Remove `remote-core/src/packet.rs` and `remote-core/src/discovery.rs`; empty `remote-core/src/lib.rs` (crate doc-comment only); drop the now-unused `serde`/`serde_json` dependencies from `remote-core/Cargo.toml`.

- [ ] **Step 5: Verify build**

Run: `cargo build && cargo clippy`
Expected: all three crates build with zero warnings; `cargo test` runs 0 tests.

---

### Task 2: Protocol — status, transport, discovery

**Files:**
- Modify: `docs/003-protocol.md` (header block; §5 Transport Architecture; §6 Ports; §8 Bluetooth Transport; §10–§17 Discovery; §159 Wire Format Summary; §166 Final Architecture diagram; §167 End State)

**Interfaces:**
- Produces: `003-protocol.md` with status `Draft`, QUIC-only v1, and a fully defined discovery wire format. Tasks 3–4 edit other sections of the same file.

- [ ] **Step 1: Header.** Change `**Status:** Stable Specification` to `**Status:** Draft`. Remove `Bluetooth discovery` from the Discovery Transport line. Add below the header table: a one-paragraph "Path to Stable" note — the document becomes Stable after two independent implementations interoperate over every P0 service.

- [ ] **Step 2: Remove Bluetooth from v1.** Delete §8 (Bluetooth Transport) and Bluetooth rows/mentions in §5, §159, and the §166 diagram. Add a short section "Future Transports" (after the transport abstraction section) stating: Bluetooth (BLE discovery + L2CAP CoC) is deferred to Protocol v2; any future non-TLS transport must define session encryption before adoption; the transport abstraction (§9) is retained for this purpose.

- [ ] **Step 3: Discovery wire format.** In the discovery sections, define a shared datagram header preceding the CBOR payload:

```text
┌──────────────┬─────────────┬──────────────┐
│ magic        │ packet type │ CBOR payload │
│ "RMT1" 4 B   │ u8          │ N bytes      │
└──────────────┴─────────────┴──────────────┘

0x01 ADVERTISEMENT
0x02 DISCOVER
```

`DISCOVER` has an empty CBOR map `{}` as payload. Datagrams whose magic or type is unknown are silently dropped.

- [ ] **Step 4: IPv6 group.** In §12, replace `ff02::1` with the dedicated link-local group `ff02::524d:5431`, keeping port 48621 and the interface-scope requirement.

- [ ] **Step 5: Privacy note.** Add a "Discovery Privacy" section after Discovery Security: the persistent `device_id` is broadcast in clear text on the local network and can be used to track the device across networks; rotating discovery identifiers are a planned v2 mitigation; users can disable discovery (per FR-001).

- [ ] **Step 6: Verify**

Run: `grep -niE 'bluetooth|l2cap|ble\b|ff02::1([^0-9a-f]|$)' docs/003-protocol.md`
Expected: matches only inside the "Future Transports" section; no bare `ff02::1` remains.

---

### Task 3: Protocol — security model (mTLS + pinning + SAS pairing)

**Files:**
- Modify: `docs/003-protocol.md` (§18–§29 Identity/Pairing; §30–§33 Session; §45 Core Message IDs; §51–§55 AUTH; §32 Session States; §136 Reconnection; §141 Security Requirements; §162 Protocol Invariants; §163 Compatibility)

**Interfaces:**
- Consumes: Task 2's revised file (same document).
- Produces: the revised session flow and PAIR_* message definitions that Task 4's registry lists must match exactly: `PAIR_REQUEST 0x0010`, `PAIR_CHALLENGE 0x0011`, `PAIR_RESPONSE 0x0012`, `PAIR_ACCEPT 0x0013`, `PAIR_REJECT 0x0014`.

- [ ] **Step 1: Identity sections.** Rewrite §19–§20: one Ed25519 key pair per installation, wrapped in a **self-signed X.509 certificate** used for QUIC/TLS 1.3; the certificate is the authentication identity; `device_id` (UUIDv4) remains the stable installation identifier. Identity storage now lists: `device_id`, `private_key`, `certificate`, `paired_devices`; paired records store `device_id`, `device_name`, `certificate` (or SPKI SHA-256), `permissions`, `paired_at`, `last_seen`.

- [ ] **Step 2: Authentication = mTLS with pinning.** Replace §51–§55 (AUTH, challenge, signature, AUTH_RESPONSE, failure) with a single "Authentication" section: both peers present certificates in the QUIC handshake (client certificate required); a peer is authenticated when the presented certificate's **SPKI SHA-256** equals the pinned value from pairing; mismatch closes the session with error `PINNING_MISMATCH`; unknown certificates may only proceed into the pairing flow and cannot reach privileged services.

- [ ] **Step 3: Pairing flow.** Rewrite §21–§28 around SAS verification. Flow diagram:

```text
Device A (initiator)              Device B (responder)
   │                                  │
   │──── PAIR_REQUEST ───────────────►│
   │                                  │
   │◄──── PAIR_CHALLENGE ─────────────│
   │                                  │
   │   Both devices display SAS       │
   │   User confirms on both          │
   │                                  │
   │──── PAIR_RESPONSE ──────────────►│
   │                                  │
   │◄──── PAIR_ACCEPT / PAIR_REJECT ──│
```

Payload tables (integer CBOR keys):

```text
PAIR_REQUEST   {1: device_id (UUID), 2: device_name (tstr), 3: nonce (bstr 32)}
PAIR_CHALLENGE {1: device_id (UUID), 2: nonce (bstr 32)}
PAIR_RESPONSE  {1: sas_confirmed (bool)}
PAIR_ACCEPT    {}
PAIR_REJECT    {1: reason (u16 error code)}
```

SAS derivation (its own section):

```text
input = "REMOTE-PAIR-V1"
     || sort(SPKI_SHA256_A, SPKI_SHA256_B)   (lexicographic, 32 B each)
     || nonce_initiator (32 B)
     || nonce_responder (32 B)

SAS = SHA-256(input) interpreted big-endian, mod 10^6,
      rendered as 6 decimal digits (zero-padded)
```

Both sides compute and display the SAS; the user must confirm equality on **both** devices; `PAIR_RESPONSE.sas_confirmed=false` or `PAIR_REJECT` aborts with no pairing record. On `PAIR_ACCEPT`, both sides pin the peer certificate. Keep the revocation section (§29), replacing `public_key` with `certificate` in the deleted-data list. Add a note: QR pairing (future) transports the SPKI fingerprint and replaces visual comparison.

- [ ] **Step 4: Session establishment and states.** Rewrite §30's flow:

```text
QUIC CONNECT (TLS 1.3, both certificates)
     ↓
HELLO ↔ HELLO_RESPONSE
     ↓
paired? ──no──► PAIR flow (user confirmation) ──accepted──┐
     │yes                                                  │
     ▼◄────────────────────────────────────────────────────┘
CAPABILITIES ↔ CAPABILITIES_RESPONSE
     ↓
PERMISSIONS ↔ PERMISSIONS_RESPONSE      (each side sends its grants)
     ↓
SESSION_READY (each side; established when both sent and received)
```

§32 states become: `DISCONNECTED → CONNECTING → CONNECTED → HANDSHAKING → PAIRING (only if unpaired) → NEGOTIATING → ESTABLISHED → CLOSING → DISCONNECTED`. Update §136 Reconnection's message list (drop AUTH). In §33, replace "after authentication" language with "after the session is established".

- [ ] **Step 5: Security requirements & invariants.** §141: replace "Authenticate paired devices" with mTLS/pinning wording; add "Require user SAS confirmation for pairing". §162 invariants: rewrite items 1–2 and 10–12 to the new model (no session without mutual certificate verification; unknown certificates confined to pairing; SAS nonces never reused; private keys never leave the device; discovery never trusted). §163 Compatibility list: remove `AUTH`/`AUTH_RESPONSE`, add the five `PAIR_*` messages.

- [ ] **Step 6: Error codes.** In §129: keep `0x0005 AUTH_FAILED` (now "pairing/authentication failure"), keep `0x0012 INVALID_CHALLENGE` (pairing nonce misuse), keep `0x0011 INVALID_SIGNATURE` but re-describe it for certificate validation failures, and append `0x001B PINNING_MISMATCH` and `0x001C PAIRING_REJECTED`.

- [ ] **Step 7: Verify**

Run: `grep -nE 'AUTH[^_A-Z]|AUTH_RESPONSE|challenge' docs/003-protocol.md`
Expected: no session-auth challenge-response remains; `challenge`/nonce mentions only inside pairing sections; `AUTH_FAILED` only in the error-code registry.

---

### Task 4: Protocol — message registries, streams, sharing, key codes

**Files:**
- Modify: `docs/003-protocol.md` (§35, §44–§45; §56–§57 capability/service IDs; §61 permissions; §65–§75 services/streams; §76–§128 per-service sections; §160–§161 registries; final renumbering pass)

**Interfaces:**
- Consumes: Task 3's PAIR_* IDs (0x0010–0x0014) in the core registry.
- Produces: the complete wire registry that `001`/`002` reference. Sharing service `0x0C00`, capability `0x000E SHARING`, permission `0x00B0 SHARE_RECEIVE`.

- [ ] **Step 1: Namespacing rule.** Add to the envelope/message-ID sections: the `type: u16` field is interpreted **within the namespace of the `service` field**; each service defines its own types starting at 0x0001. Core (service 0x0000) registry becomes:

```text
0x0001 HELLO                    0x0002 HELLO_RESPONSE
0x0010 PAIR_REQUEST             0x0011 PAIR_CHALLENGE
0x0012 PAIR_RESPONSE            0x0013 PAIR_ACCEPT
0x0014 PAIR_REJECT
0x0020 CAPABILITIES             0x0021 CAPABILITIES_RESPONSE
0x0030 PERMISSIONS              0x0031 PERMISSIONS_RESPONSE
0x0040 SESSION_READY            0x0041 SESSION_CLOSE
0x0050 PING                     0x0051 PONG
0x0060 SERVICE_START            0x0061 SERVICE_ACCEPT
0x0062 SERVICE_REJECT           0x0063 SERVICE_STOP
0x0070 STREAM_OPEN              0x0071 STREAM_ACCEPT
0x0072 STREAM_REJECT            0x0073 STREAM_CLOSE
0x0090 ERROR
```

Drop generic `0x0080 EVENT` (typed events carry the EVENT flag instead). Update §35's control-stream message list accordingly.

- [ ] **Step 2: Per-service type tables.** In each service section, add its registry and give every payload an integer-key table. Registries:

```text
CLIPBOARD (0x0100):  0x0001 CLIPBOARD_UPDATE  0x0002 CLIPBOARD_REQUEST  0x0003 CLIPBOARD_RESPONSE
FILE_TRANSFER (0x0200): 0x0001 FILE_OFFER  0x0002 FILE_ACCEPT  0x0003 FILE_REJECT
                        0x0004 FILE_COMPLETE  0x0005 FILE_CANCEL
INPUT (0x0300):  0x0001 MOUSE_MOVE  0x0002 MOUSE_BUTTON  0x0003 MOUSE_SCROLL
                 0x0004 KEY_DOWN  0x0005 KEY_UP  0x0006 TEXT_INPUT
WINDOW (0x0400): 0x0001 WINDOW_LIST  0x0002 WINDOW_LIST_RESPONSE  0x0003 WINDOW_FOCUS
                 0x0004 WINDOW_MINIMIZE  0x0005 WINDOW_MAXIMIZE  0x0006 WINDOW_RESTORE
                 0x0007 WINDOW_MOVE  0x0008 WINDOW_RESIZE  0x0009 WINDOW_CLOSE
                 0x0010 WINDOW_CHANGED (event)
MEDIA (0x0500):  0x0001 PLAY  0x0002 PAUSE  0x0003 STOP  0x0004 NEXT  0x0005 PREVIOUS
                 0x0006 SEEK  0x0007 SET_VOLUME  0x0008 SET_MUTE  0x0009 MEDIA_STATE_REQUEST
                 0x0010 PLAYBACK_CHANGED (event)  0x0011 TRACK_CHANGED (event)
                 0x0012 VOLUME_CHANGED (event)  0x0013 MUTE_CHANGED (event)
                 0x0014 MEDIA_STATE_RESPONSE
AUDIO (0x0600), VIDEO (0x0700), CAMERA (0x0800), MICROPHONE (0x0900):
                 no control-stream message types in v1; negotiation happens via
                 SERVICE_START options and STREAM_OPEN options (state this explicitly)
NOTIFICATIONS (0x0A00): 0x0001 NOTIFICATION_POST  0x0002 NOTIFICATION_DISMISS  0x0003 NOTIFICATION_CLEAR
DEVICE (0x0B00): 0x0001 DEVICE_INFO_REQUEST  0x0002 DEVICE_INFO_RESPONSE
                 0x0003 BATTERY_CHANGED (event)  0x0004 POWER_REQUEST
SHARING (0x0C00): 0x0001 SHARE_TEXT  0x0002 SHARE_URL
```

For payloads currently shown with named fields (media state, window model, notifications, device info, file messages), convert to integer-key tables numbered in the order the fields are listed today.

- [ ] **Step 3: Sharing service section.** Add after Notifications: service `0x0C00`; capability `0x000E SHARING` appended to §56/§161; permission `0x00B0 SHARE_RECEIVE` appended to §61. Payloads: `SHARE_TEXT {1: text (tstr)}`, `SHARE_URL {1: url (tstr)}`. Max payload 64 KiB; requires `SHARE_RECEIVE` permission on the receiver. Mark SHARING as an optional v1 service in §164.

- [ ] **Step 4: Commands note.** Add one line to §164 (or a short "Deferred Services" note): a Commands service is deliberately out of Protocol v1; service ID `0x0D00` is reserved for it in v2.

- [ ] **Step 5: Stream correlation.** In §71–§74: `STREAM_OPEN` payload becomes `{1: stream_id (UUID), 2: service_id, 3: stream_type, 4: direction, 5: options}`; `STREAM_ACCEPT`/`STREAM_REJECT` echo `{1: stream_id, ...}`. The data-plane QUIC stream begins with the 16 raw bytes of `stream_id`, binding it to the negotiation. FILE streams carry `{transfer_id, file_id}` inside `options`. Update §88 (file data stream) to mention the 16-byte prefix before the first chunk.

- [ ] **Step 6: Direction semantics.** In §66 add: directions are from the perspective of the peer sending `SERVICE_START`/`STREAM_OPEN`; `SEND` means the requester transmits payload data to the peer.

- [ ] **Step 7: Key codes.** In §97: `key_code` values are **USB HID Usage IDs from the Keyboard/Keypad page (0x07)** (normative reference to the USB HID Usage Tables); media/consumer keys are v2 (Consumer page 0x0C noted as reserved).

- [ ] **Step 8: Renumber and closing claims.** Renumber all sections sequentially. In §167 (End State) remove the bold "contains no undefined wire-level behavior" sentence; replace with: remaining open items (Bluetooth, Commands, clipboard history, adaptive-stream renegotiation, rotating discovery IDs) are deferred to v2.

- [ ] **Step 9: Verify**

Run: `grep -nE '0x0080 EVENT|Wire Identifier' docs/003-protocol.md && grep -c '^# ' docs/003-protocol.md`
Expected: no generic EVENT registry entry; heading count matches a clean sequential numbering (spot-check first/last sections manually).

---

### Task 5: Requirements — priorities and scope alignment

**Files:**
- Modify: `docs/001-requirements.md`

**Interfaces:**
- Consumes: protocol scope from Tasks 2–4 (Sharing in v1, Commands out, Bluetooth future, SAS pairing).

- [ ] **Step 1: Priority tags.** Add a `**Priority:** Px` line under every requirement heading, per this assignment:
  - **P0:** FR-001…FR-008, FR-CLP-001…005, FR-FILE-001/002/004/005/006/009, FR-CAP-001…004, FR-TRANSPORT-001/002, FR-CLI-001…006, FR-CONFIG-001/002, all SEC-*, NFR-PRIV-*, NFR-REL-001/003, NFR-OBS-*, NFR-MAINT-*, NFR-COMPAT-*, PLAT-001/004/005/007
  - **P1:** FR-FILE-003/007/008, FR-INPUT-001…004, FR-MEDIA-001…003, FR-SHARE-001…003, NFR-PERF-001/002/003/005, NFR-REL-002/004, PLAT-003 (Android), PLAT-006 (X11)
  - **P2:** FR-AUDIO-*, FR-VIDEO-*, FR-CAM-*, FR-MIC-*, FR-WIN-*, FR-NOTIFY-*, FR-STATUS-*, FR-GUI-*, NFR-PERF-004, PLAT-002 (Windows)
  - **P3:** FR-CLP-006 (history), FR-CMD-001…003, FR-TRANSPORT-003 (Bluetooth), FR-EXT-005 (plugins)
  - FR-TRANSPORT-004, FR-EXT-001…004: P0 (they constrain architecture now).

- [ ] **Step 2: Replace §32.** Keep the P0–P3 definitions but replace "Requirements shall eventually be assigned" with "Each requirement above carries its priority tag" and a sentence noting P0 is the first end-to-end milestone.

- [ ] **Step 3: Commands & Bluetooth.** Under FR-CMD-* add a note: out of Protocol v1 scope, deferred to v2 (service ID reserved). Under FR-TRANSPORT-003: Bluetooth is a future transport; v1 operates over QUIC only.

- [ ] **Step 4: Security updates.** Rewrite SEC-002 to "Remote shall mutually authenticate devices using TLS with pinned per-device certificates." Add:

```markdown
## SEC-011 — Pairing Verification

Pairing shall display a short verification code derived from both device
identities on both devices, and shall require the user to confirm the code
matches before trust is established.
```

- [ ] **Step 5: Success criteria.** Reorder §31 so items map to priorities: 1–6 = P0 path (install → discover → pair with code confirmation → see capabilities → transfer files → sync clipboard), then input/media (P1), then streaming items (P2); keep the DE-independence/CLI/service-isolation items at the end. Add "with code confirmation" to the pairing item.

- [ ] **Step 6: Verify**

Run: `grep -c '\*\*Priority:\*\*' docs/001-requirements.md`
Expected: count equals the number of requirement headings (`grep -cE '^## (FR|NFR|SEC|PLAT)-' docs/001-requirements.md`).

---

### Task 6: Architecture — naming and scope alignment

**Files:**
- Modify: `docs/002-architecture.md`

**Interfaces:**
- Consumes: crate names from Task 1; protocol scope from Tasks 2–4.

- [ ] **Step 1: Crate names.** Replace throughout: `connect` → `remote` (daemon), `connectcli` → `remote-cli`, `connectcore` → `remote-core`; §5, §6, §7, §8, §29 headings and bodies; §29's workspace tree updated to the new directory names (and note the daemon is now a workspace member). CLI section notes the binary is `remote`; daemon sections note its binary is `remoted`.

- [ ] **Step 2: Services lists.** In §10 and any service enumerations: add `Sharing`, remove `Commands` (add one line: commands deferred to v2). Keep the rest.

- [ ] **Step 3: Transports.** In §15 and the §40 summary diagram: Wi-Fi/QUIC is the v1 transport; label Bluetooth as `Bluetooth (future)` where it appears; keep the transport-abstraction rationale.

- [ ] **Step 4: Security model naming.** In §22–§24: pairing establishes trust by **pinning the peer's TLS certificate after user SAS confirmation**; sessions are mutually authenticated via mTLS; permissions remain independent of pairing (unchanged). One sentence each — details live in `003`.

- [ ] **Step 5: Verify**

Run: `grep -niE 'connectcore|connectcli|\bconnect\b' docs/002-architecture.md`
Expected: no matches (allow the word "connection"/"connected" — the regex above word-bounds bare `connect`).

---

### Task 7: Vision — light touch

**Files:**
- Modify: `docs/000-vision.md`

- [ ] **Step 1: Bluetooth framing.** In §6 (Connectivity) and §5 summary mentions: keep Bluetooth as a possible transport but add one sentence that the initial implementation targets Wi-Fi/QUIC only, with Bluetooth as a future transport. §1's "keeping Bluetooth as a possible transport" phrasing may stay.

- [ ] **Step 2: Naming.** §14's example tree and any crate mentions (there are none today — verify) use `remote-core`-style names if present.

- [ ] **Step 3: Verify**

Run: `grep -niE 'connect(core|cli)' docs/000-vision.md`
Expected: no matches.

---

### Task 8: Cross-doc consistency sweep and CLAUDE.md update

**Files:**
- Modify: `CLAUDE.md`; possibly small fixes in any of the four docs.

**Interfaces:**
- Consumes: everything above.

- [ ] **Step 1: Sweep.** Run and resolve every unexpected hit:

```bash
grep -rniE 'connectcore|connectcli' docs/ CLAUDE.md
grep -rniE 'AUTH_RESPONSE|0x0080 EVENT' docs/
grep -rniE 'bluetooth' docs/000-vision.md docs/001-requirements.md docs/002-architecture.md   # each hit must say future/v2
grep -rn 'Stable Specification' docs/
```

Cross-check by hand: every service/capability/permission named in `001` and `002` exists in `003`'s registries (including SHARING) and vice versa; priorities in `001` match the P0 scope described in `003`'s required-services section (§164: required = CORE, CLIPBOARD, FILE_TRANSFER; INPUT and MEDIA move from "required" to "optional, P1" to match the P0 decision — fix §164 if Task 4 left INPUT/MEDIA as required).

- [ ] **Step 2: Update CLAUDE.md.** The crate-rename and discovery-stub wording was already updated when Task 1 landed. Remaining stale part: the `003-protocol.md` bullet still says "currently **under revision**" — once Tasks 2–7 are done, rewrite it to say the doc is the revised Draft spec, keeping the pointer to `docs/design/2026-08-08-docs-revision-design.md` as design rationale. Keep the KDE-style `packet.rs` caveat (unless the file was deleted by then), architecture rules, and the working agreements unchanged.

- [ ] **Step 3: Final verify**

Run: `cargo build && cargo test && grep -rn 'Stable Specification' docs/`
Expected: build/tests pass; no "Stable Specification" anywhere; report the acceptance-criteria checklist from the spec (§10) with each item confirmed.
