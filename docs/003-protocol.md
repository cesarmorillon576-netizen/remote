# Remote Protocol Specification

**Project:** Remote
**Protocol:** Remote Protocol
**Protocol Version:** `1`
**Wire Identifier:** `remote/1`
**Status:** Draft
**Primary Transport:** QUIC over UDP
**Discovery Transport:** UDP multicast
**Control Serialization:** CBOR
**Binary Integer Encoding:** Big Endian
**Identity:** UUIDv4 + Ed25519 in a self-signed X.509 certificate
**File Integrity:** SHA-256
**Audio Codec:** Opus
**Video Codec:** H.264

**Path to Stable:** This document becomes a Stable Specification when two independent implementations interoperate over every P0 service (discovery, pairing, session establishment, capability and permission negotiation, clipboard, file transfer). Until then, wire-level details may change without a protocol version bump.

---

# 1. Purpose

Remote is a cross-platform device integration protocol designed to connect computers and mobile devices.

Remote provides bidirectional communication between devices for:

* Device discovery.
* Secure pairing.
* Clipboard synchronization.
* File transfer.
* Mouse control.
* Keyboard control.
* Window control.
* Multimedia control.
* Audio transmission.
* Video transmission.
* Camera streaming.
* Microphone streaming.
* Notifications.
* Device information.
* Device power operations where supported.

Remote is an independent protocol.

Remote does not depend on KDE Connect and does not implement KDE Connect's wire protocol.

---

# 2. Protocol Goals

Remote Protocol v1 provides:

1. Secure device authentication.
2. Bidirectional communication.
3. Capability negotiation.
4. Permission-controlled services.
5. Low-latency control messages.
6. Reliable file transfer.
7. Real-time audio and video streams.
8. Cross-platform operation.
9. Transport independence.
10. Explicit protocol versioning.
11. Deterministic message formats.
12. Service isolation.
13. Resistance to synchronization loops.
14. Graceful connection recovery.

---

# 3. Terminology

| Term       | Meaning                             |
| ---------- | ----------------------------------- |
| Device     | A Remote installation               |
| Peer       | The other device in a session       |
| Initiator  | Device initiating a connection      |
| Responder  | Device accepting a connection       |
| Session    | Authenticated Remote connection     |
| Service    | Functional Remote subsystem         |
| Stream     | QUIC stream carrying service data   |
| Capability | Feature supported by a device       |
| Permission | User authorization for a capability |
| Pairing    | Establishing trust between devices  |
| Transfer   | File transfer operation             |
| Event      | Asynchronous protocol message       |
| Request    | Message expecting a response        |
| Response   | Message associated with a request   |

---

# 4. Protocol Architecture

Remote is divided into:

```text
┌──────────────────────────────────────────┐
│                Services                  │
├──────────────────────────────────────────┤
│             Remote Protocol              │
├──────────────────────────────────────────┤
│                 Session                  │
├──────────────────────────────────────────┤
│                Transport                 │
├──────────────────────────────────────────┤
│                 Network                  │
└──────────────────────────────────────────┘
```

Services:

```text
CORE
CLIPBOARD
FILE_TRANSFER
INPUT
WINDOW
MEDIA
AUDIO
VIDEO
CAMERA
MICROPHONE
NOTIFICATIONS
DEVICE
SHARING
```

---

# 5. Transport Architecture

Protocol v1 defines a single transport:

```text
QUIC over UDP
```

QUIC provides:

* Encryption.
* Reliable streams.
* Stream multiplexing.
* Congestion control.
* Connection management.
* Low-latency transport.

Remote does not implement:

* TCP.
* Custom retransmission.
* Custom encryption.
* Custom congestion control.

---

# 6. IP Ports

Remote uses:

```text
UDP 48621 → Discovery
UDP 48622 → QUIC
```

These ports are configurable.

The protocol version does not change when ports are changed.

---

# 7. QUIC Configuration

Remote QUIC connections use:

```text
ALPN: remote/1
TLS: 1.3
```

The QUIC implementation must reject a connection that does not negotiate:

```text
remote/1
```

---

# 8. Transport Abstraction

The Remote core operates on an abstract transport.

Conceptually:

```rust
trait Transport {
    connect();
    send();
    receive();
    close();
}
```

The protocol layer must not depend directly on:

```text
Wi-Fi
Ethernet
Bluetooth
Linux
Windows
Android
```

---

# 9. Future Transports

Bluetooth (BLE discovery + L2CAP CoC byte transport) is deferred to Protocol v2.

Any future transport that does not provide TLS 1.3 natively must define session encryption and authentication before adoption.

The transport abstraction (§8) is retained for this purpose: adding a transport must not change protocol messages or services.

---

# 10. Discovery

Discovery allows devices to locate Remote peers.

Discovery:

* Does not authenticate devices.
* Does not establish sessions.
* Does not transmit private information.
* Does not grant permissions.

Discovery only advertises enough information to establish a connection.

Every discovery datagram shares one wire format:

```text
┌──────────────┬─────────────┬──────────────┐
│ magic        │ packet type │ CBOR payload │
│ "RMT1" 4 B   │ u8          │ N bytes      │
└──────────────┴─────────────┴──────────────┘
```

Packet types:

```text
0x01 ADVERTISEMENT
0x02 DISCOVER
```

A datagram whose magic or packet type is unknown is silently dropped.

---

# 11. IPv4 Discovery

Remote uses IPv4 multicast:

```text
Address: 239.255.42.21
Port:    48621
Protocol: UDP
```

Discovery packets are individual UDP datagrams.

---

# 12. IPv6 Discovery

Remote uses the dedicated link-local IPv6 multicast group:

```text
ff02::524d:5431
```

on UDP port:

```text
48621
```

Implementations must use the local interface scope.

---

# 13. Discovery Advertisement

Devices periodically announce themselves.

An advertisement is a discovery datagram with packet type `0x01` and the following CBOR payload:

```text
{
    version,
    device_id,
    device_name,
    device_type,
    platform,
    implementation_version,
    connection_port
}
```

Exact CBOR representation:

```text
{
    1: 1,
    2: device_id,
    3: device_name,
    4: device_type,
    5: platform,
    6: implementation_version,
    7: 48622
}
```

Field IDs:

```text
1 version
2 device_id
3 device_name
4 device_type
5 platform
6 implementation_version
7 connection_port
```

---

# 14. Discovery Interval

A device sends an advertisement every:

```text
5 seconds
```

A device may send an immediate advertisement when:

* Remote starts.
* Network interface becomes available.
* Device changes network.
* Device name changes.
* Connection port changes.

---

# 15. Discovery Timeout

A discovered device is considered unavailable after:

```text
15 seconds
```

without receiving an advertisement.

The device remains in the discovery cache until the timeout expires.

---

# 16. Discovery Request

A device may request an immediate advertisement using:

```text
DISCOVER
```

`DISCOVER` is a discovery datagram with packet type `0x02` and an empty CBOR map `{}` as payload.

The request is sent to:

```text
239.255.42.21:48621
```

The receiving device sends its advertisement directly to the sender.

---

# 17. Discovery Security

Discovery packets are considered untrusted.

Implementations must never trust:

* Device names.
* Platform strings.
* Implementation versions.
* Connection ports.
* Device IDs.

All values must be validated.

Discovery never grants access.

## 17.1 Discovery Privacy

The persistent `device_id` is broadcast in clear text on the local network.

An observer on the same network can use it to track the device across networks over time.

Rotating discovery identifiers are a planned v2 mitigation.

Users can disable discovery entirely (see FR-001 in the requirements document).

---

# 18. Device Identity

Every Remote installation has a persistent:

```text
device_id
```

The identifier is:

```text
UUIDv4
```

The device ID identifies the Remote installation.

It is not:

* An IP address.
* A MAC address.
* A session ID.
* A public key.
* A password.

---

# 19. Identity Key Pair

Every Remote installation generates exactly one:

```text
Ed25519 key pair
```

wrapped in a:

```text
self-signed X.509 certificate
```

The certificate is used for QUIC/TLS 1.3 and is the device's authentication identity.

The private key is generated locally.

The private key never leaves the device.

There is no separate pairing key: the same certificate is presented in every connection and pinned during pairing.

---

# 20. Identity Storage

A Remote installation stores:

```text
device_id
private_key
certificate
paired_devices
```

Paired device records contain:

```text
device_id
device_name
certificate (or its SPKI SHA-256 fingerprint)
permissions
paired_at
last_seen
```

---

# 21. Pairing

Pairing establishes permanent trust between two devices.

Pairing runs inside an established TLS session: the channel is already encrypted, but the peer is not yet trusted.

A device begins as:

```text
UNPAIRED
```

After successful pairing:

```text
PAIRED
```

A connection from an unpaired device can only proceed into the pairing flow and cannot access privileged services.

---

# 22. Pairing Requirements

Pairing requires explicit user confirmation on **both** devices.

The user must be able to identify:

```text
device_name
device_id
```

of the device requesting pairing.

The user must verify that the short authentication string (SAS) displayed on both devices is identical.

A device must not silently pair with another device.

---

# 23. Pairing Flow

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
   │                                  │
   │            PAIRED                │
```

---

# 24. Pair Request

Sent by the initiator.

Payload:

```text
{
    1: device_id,
    2: device_name,
    3: nonce
}
```

Where:

```text
device_id: UUID
device_name: UTF-8 string
nonce: 32 random bytes
```

The nonce must be freshly generated for every pairing attempt.

---

# 25. Pair Challenge

Sent by the responder.

Payload:

```text
{
    1: device_id,
    2: nonce
}
```

Where:

```text
device_id: UUID
nonce: 32 random bytes
```

The nonce must be freshly generated for every pairing attempt.

After this message, both devices can derive the SAS.

---

# 26. Short Authentication String

Both devices compute:

```text
input = "REMOTE-PAIR-V1"
     || sort(SPKI_SHA256_initiator, SPKI_SHA256_responder)
     || nonce_initiator
     || nonce_responder

SAS = SHA-256(input) interpreted as Big Endian, mod 10^6,
      rendered as 6 decimal digits (zero-padded)
```

Where:

```text
SPKI_SHA256_*: SHA-256 over the peer certificate's SubjectPublicKeyInfo (32 bytes each)
sort: lexicographic byte order
nonce_*: the 32-byte nonces from PAIR_REQUEST and PAIR_CHALLENGE
```

Each certificate fingerprint is taken from the certificate presented in the TLS handshake, not from any application-level message.

Both devices display the SAS.

The user must confirm on both devices that the codes are identical.

Matching codes prove that both devices see the same two certificates: a man-in-the-middle would cause the codes to differ.

---

# 27. Pair Acceptance

After the user confirms the SAS, the initiator sends:

```text
PAIR_RESPONSE
{
    1: sas_confirmed
}
```

with `sas_confirmed = true`.

If the responder's user also confirmed, the responder sends:

```text
PAIR_ACCEPT
{}
```

On PAIR_ACCEPT, both devices pin the peer certificate (store its SPKI SHA-256 fingerprint) and persist the pairing record.

QR-code pairing (future) transports the SPKI fingerprint directly and replaces the visual comparison.

---

# 28. Pair Rejection

If the user rejects pairing, or the SAS codes do not match:

* The initiator sends `PAIR_RESPONSE` with `sas_confirmed = false`, or
* The responder sends:

```text
PAIR_REJECT
{
    1: reason
}
```

where `reason` is a u16 error code.

In both cases the connection is closed and no pairing record is created.

---

# 29. Pair Revocation

A paired device may be revoked locally.

Revocation deletes:

```text
certificate (pinned fingerprint)
permissions
pairing metadata
```

from the local trust store.

A revoked device must pair again before accessing privileged services.

---

# 30. Session Establishment

```text
QUIC CONNECT (TLS 1.3, both certificates presented)
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

For a paired peer, authentication happens during the QUIC handshake via certificate pinning (§51); no application-level authentication messages exist.

---

# 31. Session ID

Each session has a randomly generated:

```text
UUIDv4
```

The session ID is generated by the initiator.

The session ID is unique to the current connection.

A session ID is never reused.

---

# 32. Session States

```text
DISCONNECTED
     ↓
CONNECTING
     ↓
CONNECTED
     ↓
HANDSHAKING
     ↓
PAIRING          (only if the peer is not paired)
     ↓
NEGOTIATING
     ↓
ESTABLISHED
     ↓
CLOSING
     ↓
DISCONNECTED
```

A connection with a paired peer skips `PAIRING` and moves directly from `HANDSHAKING` to `NEGOTIATING`.

---

# 33. Connection Roles

The QUIC client is the:

```text
initiator
```

The QUIC server is the:

```text
responder
```

These roles do not determine permissions.

Both devices are equal protocol peers after the session is established.

Either device may:

* Send requests.
* Send events.
* Start services.
* Transfer files.
* Control the other device.
* Create streams.

---

# 34. Control Stream

The first bidirectional QUIC stream is:

```text
Stream ID 0
```

It is the Remote control stream.

The control stream carries all session-management messages.

---

# 35. Control Stream Messages

The control stream carries:

```text
HELLO
HELLO_RESPONSE

PAIR_REQUEST
PAIR_CHALLENGE
PAIR_RESPONSE
PAIR_ACCEPT
PAIR_REJECT

CAPABILITIES
CAPABILITIES_RESPONSE

PERMISSIONS
PERMISSIONS_RESPONSE

SESSION_READY
SESSION_CLOSE

PING
PONG

SERVICE_START
SERVICE_ACCEPT
SERVICE_REJECT
SERVICE_STOP

STREAM_OPEN
STREAM_ACCEPT
STREAM_REJECT
STREAM_CLOSE

ERROR
```

plus every service-scoped message defined by the per-service registries.

Large binary data must not use the control stream.

---

# 36. CBOR Serialization

Remote control messages use:

```text
CBOR
```

CBOR maps use integer keys.

This minimizes message size.

Text strings are UTF-8.

Binary values use CBOR byte strings.

---

# 37. Message Envelope

Every control message uses:

```text
{
    1: version,
    2: type,
    3: flags,
    4: message_id,
    5: request_id?,
    6: session_id?,
    7: service,
    8: payload
}
```

Fields:

| ID | Field      | Type | Required    |
| -: | ---------- | ---- | ----------- |
|  1 | version    | u8   | yes         |
|  2 | type       | u16  | yes         |
|  3 | flags      | u16  | yes         |
|  4 | message_id | u64  | yes         |
|  5 | request_id | u64  | no          |
|  6 | session_id | UUID | conditional |
|  7 | service    | u16  | yes         |
|  8 | payload    | CBOR | yes         |

The `type` field is interpreted **within the namespace of the `service` field**: each service defines its own message types starting at `0x0001`. The same type value means different messages under different services.

---

# 38. Message Framing

The control stream uses:

```text
┌───────────────┬─────────────────────┐
│ Length        │ CBOR Message        │
│ 4 bytes       │ N bytes             │
│ Big Endian    │                     │
└───────────────┴─────────────────────┘
```

The length field contains only the CBOR payload length.

Example:

```text
00 00 00 40
```

means:

```text
64 bytes of CBOR payload
```

---

# 39. Maximum Control Message

Maximum CBOR payload:

```text
16 MiB
```

Messages larger than this are rejected.

Large payloads must use service streams.

---

# 40. Message IDs

Each device maintains its own outgoing:

```text
message_id
```

The first message uses:

```text
1
```

Every subsequent message increments the counter.

Message IDs only need to be unique within a session and direction.

---

# 41. Requests

A request contains:

```text
message_id
```

but no:

```text
request_id
```

unless it is itself responding to another request.

---

# 42. Responses

A response contains:

```text
message_id
request_id
```

where:

```text
request_id = original request message_id
```

---

# 43. Events

Events contain:

```text
message_id
```

and no:

```text
request_id
```

Events are asynchronous.

---

# 44. Message Flags

```text
0x0001 RESPONSE
0x0002 EVENT
0x0004 ERROR
0x0008 ACK_REQUIRED
0x0010 ACK
```

Unknown flags are ignored when they are not mandatory.

---

# 45. Core Message Types

Message types for service `0x0000 CORE`:

```text
0x0001 HELLO
0x0002 HELLO_RESPONSE

0x0010 PAIR_REQUEST
0x0011 PAIR_CHALLENGE
0x0012 PAIR_RESPONSE
0x0013 PAIR_ACCEPT
0x0014 PAIR_REJECT

0x0020 CAPABILITIES
0x0021 CAPABILITIES_RESPONSE

0x0030 PERMISSIONS
0x0031 PERMISSIONS_RESPONSE

0x0040 SESSION_READY
0x0041 SESSION_CLOSE

0x0050 PING
0x0051 PONG

0x0060 SERVICE_START
0x0061 SERVICE_ACCEPT
0x0062 SERVICE_REJECT
0x0063 SERVICE_STOP

0x0070 STREAM_OPEN
0x0071 STREAM_ACCEPT
0x0072 STREAM_REJECT
0x0073 STREAM_CLOSE

0x0090 ERROR
```

There is no generic EVENT message: typed events use their own message type with the `EVENT` flag set.

---

# 46. HELLO

Service:

```text
CORE = 0x0000
```

Payload:

```text
{
    1: protocol_version,
    2: device_id,
    3: device_name,
    4: device_type,
    5: platform,
    6: implementation_version
}
```

---

# 47. Device Type

Valid values:

```text
0x01 COMPUTER
0x02 PHONE
0x03 TABLET
```

---

# 48. Platform

Valid values:

```text
0x01 LINUX
0x02 WINDOWS
0x03 ANDROID
```

---

# 49. HELLO_RESPONSE

The responder sends the same identity structure:

```text
{
    1: protocol_version,
    2: device_id,
    3: device_name,
    4: device_type,
    5: platform,
    6: implementation_version
}
```

---

# 50. Protocol Version Negotiation

Both devices advertise:

```text
protocol_version
```

Remote Protocol v1 accepts:

```text
1
```

Any other version is rejected.

---

# 51. Authentication

Authentication is mutual TLS with certificate pinning.

Both peers present their certificates during the QUIC/TLS 1.3 handshake.

The client certificate is required: a connection without one is rejected.

There are no application-level authentication messages.

Authentication is bound to the encrypted channel by construction: proving possession of the private key is part of the TLS handshake itself.

---

# 52. Certificate Pinning

A peer is authenticated when the SPKI SHA-256 fingerprint of its presented certificate equals the fingerprint pinned during pairing.

Verification is performed by both sides.

Standard certificate-chain validation (CA trust) is not used: the pinned fingerprint is the only trust anchor.

Certificate expiry dates are ignored; trust is managed through pairing and revocation.

---

# 53. SPKI Fingerprint

The fingerprint is:

```text
SHA-256 over the DER-encoded SubjectPublicKeyInfo of the certificate
```

producing 32 bytes.

Using the SPKI (rather than the whole certificate) keeps the pin stable if the certificate is ever re-issued around the same key.

---

# 54. Pinning Failure

If the presented certificate's fingerprint does not match the pinned value:

* The session is closed with error `PINNING_MISMATCH`.
* No message beyond the error is processed.
* The pairing record is not modified.

A pinning mismatch may indicate an attack or a reinstalled peer; the device must never silently re-pin.

Recovering requires the user to revoke the old pairing and pair again.

---

# 55. Unknown Certificates

A connection from a certificate that matches no pairing record is allowed to proceed only into the pairing flow (§21).

An unknown peer:

* May exchange HELLO and PAIR_* messages.
* Cannot access privileged services.
* Cannot open data streams.

---

# 56. Capability IDs

```text
0x0001 CLIPBOARD
0x0002 FILE_TRANSFER
0x0003 MOUSE
0x0004 KEYBOARD
0x0005 WINDOW_CONTROL
0x0006 MEDIA_CONTROL
0x0007 AUDIO
0x0008 VIDEO
0x0009 CAMERA
0x000A MICROPHONE
0x000B NOTIFICATIONS
0x000C DEVICE_INFO
0x000D DEVICE_POWER
0x000E SHARING
```

---

# 57. Service IDs

```text
0x0000 CORE
0x0100 CLIPBOARD
0x0200 FILE_TRANSFER
0x0300 INPUT
0x0400 WINDOW
0x0500 MEDIA
0x0600 AUDIO
0x0700 VIDEO
0x0800 CAMERA
0x0900 MICROPHONE
0x0A00 NOTIFICATIONS
0x0B00 DEVICE
0x0C00 SHARING
```

---

# 58. Capability Advertisement

The `CAPABILITIES` message contains:

```text
{
    1: [
        capability_id,
        capability_metadata
    ]
}
```

Capabilities without metadata use:

```text
null
```

---

# 59. Capability Negotiation

Each device advertises its capabilities.

The effective capabilities are:

```text
LOCAL_CAPABILITIES
∩
REMOTE_CAPABILITIES
```

A service can only be started when both devices support it.

---

# 60. Permission Model

Capabilities and permissions are independent.

A device may support:

```text
CAMERA
MICROPHONE
INPUT
```

without granting the peer access.

Permissions are evaluated when a service is started.

---

# 61. Permission IDs

```text
0x0001 CLIPBOARD_READ
0x0002 CLIPBOARD_WRITE

0x0010 FILE_RECEIVE
0x0011 FILE_SEND

0x0020 MOUSE_CONTROL
0x0021 KEYBOARD_CONTROL

0x0030 WINDOW_READ
0x0031 WINDOW_CONTROL

0x0040 MEDIA_CONTROL

0x0050 AUDIO_SEND
0x0051 AUDIO_RECEIVE

0x0060 VIDEO_SEND
0x0061 VIDEO_RECEIVE

0x0070 CAMERA
0x0080 MICROPHONE

0x0090 NOTIFICATIONS

0x00A0 DEVICE_INFO
0x00A1 DEVICE_POWER

0x00B0 SHARE_RECEIVE
```

---

# 62. Permission States

```text
DENIED
ALLOWED
```

The initial state for a newly paired device is:

```text
DENIED
```

for all privileged permissions.

---

# 63. Permission Negotiation

The `PERMISSIONS` message contains the permissions the local device grants to the peer.

Example:

```text
{
    1: [
        [0x0001, ALLOWED],
        [0x0002, ALLOWED],
        [0x0020, DENIED]
    ]
}
```

---

# 64. SESSION_READY

The session becomes established after:

```text
HELLO
AUTH
CAPABILITIES
PERMISSIONS
```

have completed successfully.

After `SESSION_READY`, services may be started.

---

# 65. Service Start

A device requests a service using:

```text
SERVICE_START
```

Payload:

```text
{
    1: service_id,
    2: service_version,
    3: direction,
    4: options
}
```

---

# 66. Direction

Valid directions:

```text
0x01 SEND
0x02 RECEIVE
0x03 BIDIRECTIONAL
```

Directions are from the perspective of the peer sending `SERVICE_START` / `STREAM_OPEN`: `SEND` means the requester transmits payload data to the peer.

---

# 67. Service Acceptance

Payload:

```text
{
    1: service_id,
    2: service_version,
    3: direction,
    4: options
}
```

---

# 68. Service Rejection

Payload:

```text
{
    1: service_id,
    2: error_code
}
```

---

# 69. Service Stop

Payload:

```text
{
    1: service_id,
    2: reason
}
```

---

# 70. Stream Architecture

QUIC streams are used for:

```text
Files
Audio
Video
Camera
Microphone
```

Control operations use the control stream.

---

# 71. Stream Opening

A service requests a data stream:

```text
STREAM_OPEN
```

Payload:

```text
{
    1: stream_id,
    2: service_id,
    3: stream_type,
    4: direction,
    5: options
}
```

`stream_id` is a UUIDv4 chosen by the requester.

The QUIC stream itself is opened by the initiating peer.

The data-plane QUIC stream begins with the 16 raw bytes of `stream_id`, binding it to the negotiated service, type, and options.

FILE streams carry `transfer_id` and `file_id` inside `options`.

---

# 72. Stream Acceptance

The peer responds:

```text
STREAM_ACCEPT
```

Payload:

```text
{
    1: stream_id,
    2: options
}
```

The QUIC stream is now active.

---

# 73. Stream Rejection

The peer sends:

```text
STREAM_REJECT
```

Payload:

```text
{
    1: stream_id,
    2: error_code
}
```

The stream is closed.

---

# 74. Stream Types

```text
0x01 FILE
0x02 AUDIO
0x03 VIDEO
0x04 CAMERA
0x05 MICROPHONE
```

---

# 75. Traffic Classes

Remote defines four traffic classes:

```text
0x01 REALTIME
0x02 INTERACTIVE
0x03 CONTROL
0x04 BULK
```

| Class       | Services                         |
| ----------- | -------------------------------- |
| REALTIME    | Audio, video, camera, microphone |
| INTERACTIVE | Mouse, keyboard                  |
| CONTROL     | Clipboard, media, windows        |
| BULK        | Files                            |

Separate QUIC streams are used to prevent bulk transfers from blocking interactive traffic.

---

# 76. Clipboard Service

Service:

```text
0x0100
```

Capabilities:

```text
0x0001
```

Supported content:

```text
text/plain
text/html
image/png
```

Message types:

```text
0x0001 CLIPBOARD_UPDATE
0x0002 CLIPBOARD_REQUEST
0x0003 CLIPBOARD_RESPONSE
```

---

# 77. Clipboard Update

Message:

```text
CLIPBOARD_UPDATE
```

Payload:

```text
{
    1: update_id,
    2: origin_device,
    3: content_type,
    4: content
}
```

---

# 78. Clipboard Update ID

`update_id` is a UUID.

The receiver stores recently processed IDs.

If an update ID has already been processed, it must not be applied again.

This prevents synchronization loops.

---

# 79. Clipboard Maximum Size

Maximum clipboard payload:

```text
16 MiB
```

Clipboard data larger than this is rejected.

---

# 80. Clipboard Request

A device may request the current clipboard:

```text
CLIPBOARD_REQUEST
```

Response:

```text
CLIPBOARD_RESPONSE
```

Payload:

```text
{
    1: update_id,
    2: content_type,
    3: content
}
```

---

# 81. File Transfer Service

Service:

```text
0x0200
```

Message types:

```text
0x0001 FILE_OFFER
0x0002 FILE_ACCEPT
0x0003 FILE_REJECT
0x0004 FILE_COMPLETE
0x0005 FILE_CANCEL
```

---

# 82. File Offer

A file transfer starts with:

```text
FILE_OFFER
```

Payload:

```text
{
    1: transfer_id,
    2: file_id,
    3: name,
    4: size,
    5: mime_type,
    6: sha256
}
```

---

# 83. File IDs

Each file in a transfer has:

```text
file_id: UUID
```

A transfer may contain multiple files.

---

# 84. Transfer ID

Each transfer has:

```text
transfer_id: UUID
```

A transfer can contain:

```text
1..N files
```

---

# 85. File Name

File names are UTF-8.

Path separators are not permitted in:

```text
name
```

Directory traversal is prohibited.

The receiver determines the final filesystem path.

---

# 86. File Acceptance

The receiver responds:

```text
FILE_ACCEPT
```

Payload:

```text
{
    1: transfer_id,
    2: file_id,
    3: offset
}
```

`offset` allows interrupted transfers to resume.

---

# 87. File Rejection

```text
FILE_REJECT
```

Payload:

```text
{
    1: transfer_id,
    2: file_id,
    3: error_code
}
```

---

# 88. File Data Stream

File contents are transmitted through a dedicated QUIC stream.

The stream begins with the 16 raw bytes of the negotiated `stream_id` (§71), before the first chunk.

Each chunk contains:

```text
offset
length
data
```

Binary format:

```text
┌────────────┬────────────┬──────────────┐
│ offset u64 │ length u32 │ data         │
└────────────┴────────────┴──────────────┘
```

---

# 89. File Completion

After transmission:

```text
FILE_COMPLETE
```

Payload:

```text
{
    1: transfer_id,
    2: file_id,
    3: sha256
}
```

The receiver verifies the SHA-256 hash.

---

# 90. File Cancellation

Either peer may send:

```text
FILE_CANCEL
```

Payload:

```text
{
    1: transfer_id,
    2: file_id,
    3: reason
}
```

---

# 91. File Integrity

A transfer is successful only when:

```text
received_size == expected_size
```

and:

```text
SHA256(received_data) == expected_sha256
```

---

# 92. Input Service

Service:

```text
0x0300
```

Capabilities:

```text
MOUSE
KEYBOARD
```

Message types:

```text
0x0001 MOUSE_MOVE
0x0002 MOUSE_BUTTON
0x0003 MOUSE_SCROLL
0x0004 KEY_DOWN
0x0005 KEY_UP
0x0006 TEXT_INPUT
```

---

# 93. Mouse Movement

Message:

```text
MOUSE_MOVE
```

Payload:

```text
{
    1: dx,
    2: dy
}
```

`dx` and `dy` are signed 32-bit integers.

Coordinates are relative.

---

# 94. Mouse Buttons

Message:

```text
MOUSE_BUTTON
```

Payload:

```text
{
    1: button,
    2: state
}
```

Buttons:

```text
0x01 LEFT
0x02 RIGHT
0x03 MIDDLE
0x04 BACK
0x05 FORWARD
```

States:

```text
0x01 DOWN
0x02 UP
```

---

# 95. Mouse Scroll

Message:

```text
MOUSE_SCROLL
```

Payload:

```text
{
    1: dx,
    2: dy
}
```

---

# 96. Keyboard

Messages:

```text
KEY_DOWN
KEY_UP
TEXT_INPUT
```

---

# 97. Keyboard Key

`KEY_DOWN` and `KEY_UP` use a logical Remote key code.

Payload:

```text
{
    1: key_code
}
```

`key_code` values are **USB HID Usage IDs from the Keyboard/Keypad page (0x07)**, as defined by the USB HID Usage Tables (normative reference).

The key code is independent of the local operating system.

Media and consumer keys are not part of v1; the HID Consumer page (0x0C) is reserved for v2.

---

# 98. Text Input

`TEXT_INPUT` sends Unicode text.

Payload:

```text
{
    1: text
}
```

Text input is distinct from physical key events.

---

# 99. Input Permissions

Mouse control requires:

```text
MOUSE_CONTROL
```

Keyboard control requires:

```text
KEYBOARD_CONTROL
```

A peer without permission receives:

```text
PERMISSION_DENIED
```

---

# 100. Window Service

Service:

```text
0x0400
```

Message types:

```text
0x0001 WINDOW_LIST
0x0002 WINDOW_LIST_RESPONSE
0x0003 WINDOW_FOCUS
0x0004 WINDOW_MINIMIZE
0x0005 WINDOW_MAXIMIZE
0x0006 WINDOW_RESTORE
0x0007 WINDOW_MOVE
0x0008 WINDOW_RESIZE
0x0009 WINDOW_CLOSE
0x0010 WINDOW_CHANGED (event)
```

---

# 101. Window Model

Each window is a CBOR map:

```text
{
    1: window_id,
    2: title,
    3: application,
    4: x,
    5: y,
    6: width,
    7: height,
    8: state
}
```

---

# 102. Window States

```text
0x01 NORMAL
0x02 MINIMIZED
0x03 MAXIMIZED
0x04 FULLSCREEN
```

---

# 103. Window Operations

```text
WINDOW_LIST
WINDOW_FOCUS
WINDOW_MINIMIZE
WINDOW_MAXIMIZE
WINDOW_RESTORE
WINDOW_MOVE
WINDOW_RESIZE
WINDOW_CLOSE
```

---

# 104. Window List

`WINDOW_LIST_RESPONSE` payload:

```text
{
    1: [window, window, ...]
}
```

using the window map from §101.

`WINDOW_CHANGED` is an event carrying a single window map:

```text
{
    1: window
}
```

---

# 105. Window Control

Window operations require:

```text
WINDOW_CONTROL
```

Window metadata can be requested using:

```text
WINDOW_READ
```

---

# 106. Media Service

Service:

```text
0x0500
```

Message types:

```text
0x0001 PLAY
0x0002 PAUSE
0x0003 STOP
0x0004 NEXT
0x0005 PREVIOUS
0x0006 SEEK
0x0007 SET_VOLUME
0x0008 SET_MUTE
0x0009 MEDIA_STATE_REQUEST
0x0010 PLAYBACK_CHANGED (event)
0x0011 TRACK_CHANGED (event)
0x0012 VOLUME_CHANGED (event)
0x0013 MUTE_CHANGED (event)
0x0014 MEDIA_STATE_RESPONSE
```

---

# 107. Media Commands

```text
PLAY
PAUSE
STOP
NEXT
PREVIOUS
SEEK
SET_VOLUME
SET_MUTE
```

---

# 108. Media Events

```text
PLAYBACK_CHANGED
TRACK_CHANGED
VOLUME_CHANGED
MUTE_CHANGED
```

---

# 109. Media State

`MEDIA_STATE_RESPONSE` (and media events) carry the media state map:

```text
{
    1: playing,
    2: title,
    3: artist,
    4: album,
    5: position,
    6: duration,
    7: volume,
    8: muted
}
```

---

# 110. Audio Service

Service:

```text
0x0600
```

The audio service defines no control-stream message types in v1: negotiation happens via `SERVICE_START` options and `STREAM_OPEN` options.

Codec:

```text
Opus
```

---

# 111. Audio Negotiation

Audio options:

```text
codec
sample_rate
channels
bitrate
frame_duration
```

Allowed sample rates:

```text
8000
12000
16000
24000
48000
```

Preferred sample rate:

```text
48000 Hz
```

---

# 112. Audio Channels

Supported:

```text
1 channel
2 channels
```

Representing:

```text
mono
stereo
```

---

# 113. Audio Stream Frame

Each audio frame contains:

```text
timestamp
sequence
payload
```

Binary structure:

```text
┌──────────────┬──────────────┬──────────────┐
│ timestamp u64│ sequence u32 │ opus payload │
└──────────────┴──────────────┴──────────────┘
```

Timestamp unit:

```text
microseconds
```

---

# 114. Audio Direction

Audio supports:

```text
SEND
RECEIVE
BIDIRECTIONAL
```

Example:

```text
Phone microphone
       ↓
     Remote
       ↓
Laptop input
```

and:

```text
Laptop output
       ↓
     Remote
       ↓
Phone speaker
```

may operate simultaneously.

---

# 115. Audio Synchronization

Every audio stream has a monotonic timestamp.

Receivers use timestamps to maintain playback timing.

Independent audio streams do not assume a shared hardware clock.

---

# 116. Video Service

Service:

```text
0x0700
```

The video service defines no control-stream message types in v1: negotiation happens via `SERVICE_START` options and `STREAM_OPEN` options.

Codec:

```text
H.264
```

---

# 117. Video Negotiation

Video options:

```text
codec
width
height
fps
bitrate
profile
level
```

Supported profiles:

```text
Baseline
Main
High
```

---

# 118. Video Frame

Each frame contains:

```text
timestamp
frame_number
flags
payload
```

Binary structure:

```text
┌──────────────┬──────────────┬─────────┬──────────────┐
│ timestamp u64│ frame u32    │ flags u8│ H.264 data   │
└──────────────┴──────────────┴─────────┴──────────────┘
```

Timestamp unit:

```text
microseconds
```

---

# 119. Video Frame Flags

```text
0x01 KEYFRAME
0x02 CONFIGURATION
0x04 END_OF_FRAME
```

---

# 120. Camera Service

Service:

```text
0x0800
```

The camera service defines no control-stream message types in v1: negotiation happens via `SERVICE_START` options and `STREAM_OPEN` options.

The camera service exposes a physical camera as a Remote video source.

Example:

```text
Phone camera
     ↓
Camera service
     ↓
Video stream
     ↓
Laptop
```

---

# 121. Camera Metadata

Camera capability metadata contains:

```text
camera_id
name
supported_resolutions
supported_framerates
supported_codecs
```

---

# 122. Camera Permission

Camera streaming requires:

```text
CAMERA
```

permission.

The device's operating system camera permission is also required.

---

# 123. Microphone Service

Service:

```text
0x0900
```

The microphone service defines no control-stream message types in v1: negotiation happens via `SERVICE_START` options and `STREAM_OPEN` options.

The microphone service exposes a physical microphone as a Remote audio source.

Example:

```text
Phone microphone
      ↓
Microphone service
      ↓
Audio stream
      ↓
Laptop
```

---

# 124. Microphone Permission

Microphone streaming requires:

```text
MICROPHONE
```

permission.

The operating system microphone permission must also be granted.

---

# 125. Notifications Service

Service:

```text
0x0A00
```

Message types:

```text
0x0001 NOTIFICATION_POST
0x0002 NOTIFICATION_DISMISS
0x0003 NOTIFICATION_CLEAR
```

`NOTIFICATION_POST` payload:

```text
{
    1: notification_id,
    2: application,
    3: title,
    4: body,
    5: timestamp
}
```

`NOTIFICATION_DISMISS` payload:

```text
{
    1: notification_id
}
```

`NOTIFICATION_CLEAR` payload:

```text
{}
```

---

# 126. Notification Permissions

Notifications require:

```text
NOTIFICATIONS
```

permission.

---

# 127. Device Service

Service:

```text
0x0B00
```

Message types:

```text
0x0001 DEVICE_INFO_REQUEST
0x0002 DEVICE_INFO_RESPONSE
0x0003 BATTERY_CHANGED (event)
0x0004 POWER_REQUEST
```

`DEVICE_INFO_RESPONSE` payload:

```text
{
    1: device_id,
    2: device_name,
    3: device_type,
    4: platform,
    5: battery_percentage,
    6: charging
}
```

`BATTERY_CHANGED` payload:

```text
{
    1: battery_percentage,
    2: charging
}
```

---

# 128. Device Power

`POWER_REQUEST` payload:

```text
{
    1: operation
}
```

Operations:

```text
0x01 LOCK
0x02 SLEEP
0x03 SHUTDOWN
0x04 RESTART
```

These operations require:

```text
DEVICE_POWER
```

permission.

The operating system may reject an operation even when Remote permission is granted.

---

# 129. Sharing Service

Service:

```text
0x0C00
```

Capability:

```text
0x000E SHARING
```

Sharing sends short text or a URL to the peer, which decides how to present it (open, copy, save).

Message types:

```text
0x0001 SHARE_TEXT
0x0002 SHARE_URL
```

`SHARE_TEXT` payload:

```text
{
    1: text
}
```

`SHARE_URL` payload:

```text
{
    1: url
}
```

Maximum payload:

```text
64 KiB
```

---

# 130. Sharing Permission

Receiving shared content requires:

```text
SHARE_RECEIVE
```

permission on the receiving device.

The receiver must never execute or navigate to shared content without user interaction.

---

# 131. Error Codes

```text
0x0001 UNKNOWN_MESSAGE
0x0002 INVALID_MESSAGE
0x0003 INVALID_STATE
0x0004 UNSUPPORTED_VERSION
0x0005 AUTH_FAILED
0x0006 PERMISSION_DENIED
0x0007 UNSUPPORTED_SERVICE
0x0008 SERVICE_UNAVAILABLE
0x0009 INVALID_PAYLOAD
0x000A RESOURCE_UNAVAILABLE
0x000B TRANSFER_FAILED
0x000C STREAM_FAILED
0x000D SESSION_EXPIRED
0x000E RATE_LIMITED
0x000F ALREADY_PAIRED
0x0010 NOT_PAIRED
0x0011 INVALID_SIGNATURE
0x0012 INVALID_CHALLENGE
0x0013 FILE_EXISTS
0x0014 FILE_NOT_FOUND
0x0015 CHECKSUM_MISMATCH
0x0016 INVALID_OFFSET
0x0017 CODEC_UNSUPPORTED
0x0018 FORMAT_UNSUPPORTED
0x0019 DEVICE_BUSY
0x001A OPERATION_CANCELLED
0x001B PINNING_MISMATCH
0x001C PAIRING_REJECTED
```

Notes:

* `AUTH_FAILED` covers pairing/authentication failures in general.
* `INVALID_SIGNATURE` covers certificate validation failures.
* `INVALID_CHALLENGE` covers pairing nonce misuse.
* `PINNING_MISMATCH` is raised when a presented certificate does not match the pinned fingerprint (§54).
* `PAIRING_REJECTED` is raised when the user rejects pairing or the SAS confirmation fails.

---

# 132. Error Message

Payload:

```text
{
    1: error_code,
    2: message,
    3: request_id?
}
```

The numeric error code determines behavior.

The text message is informational.

---

# 133. Unknown Messages

An unknown optional message must result in:

```text
UNKNOWN_MESSAGE
```

The session remains active.

---

# 134. Invalid Messages

Malformed messages result in:

```text
INVALID_MESSAGE
```

Repeated malformed messages may terminate the session.

---

# 135. Rate Limiting

Rate limiting applies independently to:

```text
Discovery
Clipboard
Input
Service creation
File offers
Control messages
```

Media streams are controlled by stream-level flow control.

---

# 136. Keepalive

Remote uses:

```text
PING
PONG
```

Payload:

```text
{
    1: timestamp
}
```

The receiver responds with the same timestamp.

QUIC transport keepalive remains handled by the QUIC implementation.

---

# 137. Session Close

Graceful termination:

```text
SESSION_CLOSE
```

Payload:

```text
{
    1: reason
}
```

Reasons:

```text
USER_REQUEST
SHUTDOWN
PROTOCOL_ERROR
AUTH_FAILURE
TRANSPORT_FAILURE
```

---

# 138. Reconnection

A lost session is considered terminated.

A new connection always performs:

```text
QUIC handshake (certificate pinning)
HELLO
CAPABILITIES
PERMISSIONS
SESSION_READY
```

The previous session ID is never reused.

---

# 139. Service Isolation

Failure of one service must not terminate the session.

Example:

```text
Camera stream fails
```

must not terminate:

```text
Clipboard
Files
Input
Media
```

Only the affected service is terminated.

---

# 140. Stream Closure

A stream can be closed independently.

Payload:

```text
{
    1: stream_id,
    2: reason
}
```

The QUIC stream is then closed.

---

# 141. Resource Limits

Implementations enforce:

```text
16 MiB maximum control message
16 MiB maximum clipboard payload
maximum concurrent file transfers: 8
maximum concurrent media streams: 16
maximum simultaneous sessions per device: 8
```

These are Remote Protocol v1 limits.

---

# 142. File Transfer Limits

Maximum individual file size:

```text
8 EiB
```

The practical filesystem limit remains platform-dependent.

---

# 143. Security Requirements

Remote implementations must:

* Use TLS 1.3 through QUIC.
* Require client certificates (mutual TLS).
* Authenticate paired devices by SPKI SHA-256 certificate pinning.
* Require user SAS confirmation on both devices for pairing.
* Protect private keys.
* Validate every message.
* Validate all payload lengths.
* Verify file hashes.
* Enforce permissions.
* Enforce resource limits.
* Reject directory traversal.
* Never trust discovery information.
* Avoid logging sensitive content.

---

# 144. Sensitive Information

The following must never be logged by default:

```text
Private keys
Pairing nonces
SAS codes
Clipboard contents
File contents
Audio payloads
Video payloads
Camera frames
Microphone data
Keyboard text
```

Debug logging may include:

```text
message type
service
message ID
session ID
payload size
timestamp
direction
```

---

# 145. Key Protection

Private identity keys must be stored using the operating system's secure storage mechanism when available.

Linux:

```text
Secret Service / secure filesystem permissions
```

Windows:

```text
Windows credential protection
```

Android:

```text
Android Keystore
```

---

# 146. Permission Principle

Remote permission and operating-system permission are independent.

An operation is allowed only when:

```text
Remote permission == ALLOWED
```

and:

```text
Operating system permission == ALLOWED
```

when the platform requires one.

---

# 147. Linux Platform Abstraction

The protocol does not depend on a specific Linux desktop environment.

Platform adapters may implement:

```text
Wayland
X11
PipeWire
PulseAudio
ALSA
```

according to platform availability.

The protocol itself remains unchanged.

---

# 148. Windows Platform Abstraction

Windows implementations use native platform APIs.

The protocol does not expose Windows-specific API names.

---

# 149. Android Platform Abstraction

Android implementations use Android APIs for:

```text
Camera
Microphone
Clipboard
Audio
Notifications
Storage
```

The Android implementation exposes them through the same Remote services.

---

# 150. CLI and GUI

Remote CLI and GUI clients use the same core protocol implementation.

Architecture:

```text
             remote-core
              /        \
             /          \
         remote     remote-gui (future)
        (remoted)
             │
         remote-cli
```

The CLI does not implement a separate protocol.

---

# 151. Protocol Independence

The protocol has no dependency on:

```text
KDE Connect
GNOME
KDE Plasma
Hyprland
Windows Explorer
Android UI
```

Operating-system integrations are adapters.

---

# 152. Clipboard Synchronization Example

```text
Phone                         Laptop
  │                              │
  │ CLIPBOARD_UPDATE             │
  │─────────────────────────────►│
  │                              │
  │                              │
  │       Laptop clipboard       │
  │                              │
```

If the laptop changes its clipboard:

```text
Laptop                        Phone
  │                              │
  │ CLIPBOARD_UPDATE             │
  │─────────────────────────────►│
  │                              │
```

The `update_id` prevents loops.

---

# 153. Camera Example

```text
Phone                         Laptop
  │                              │
  │ SERVICE_START CAMERA         │
  │─────────────────────────────►│
  │                              │
  │◄──────── SERVICE_ACCEPT ─────│
  │                              │
  │◄──────── STREAM_ACCEPT ──────│
  │                              │
  │────── H.264 stream ─────────►│
  │────── H.264 stream ─────────►│
  │────── H.264 stream ─────────►│
```

---

# 154. Microphone Example

```text
Phone                         Laptop
  │                              │
  │ SERVICE_START MICROPHONE     │
  │─────────────────────────────►│
  │                              │
  │◄──────── SERVICE_ACCEPT ─────│
  │                              │
  │────── Opus stream ──────────►│
  │────── Opus stream ──────────►│
```

---

# 155. Bidirectional Audio Example

```text
             Remote Session

Phone microphone ───────► Laptop audio input

Phone speaker     ◄────── Laptop audio output
```

Both streams may operate simultaneously.

---

# 156. File Transfer Example

```text
Laptop                         Phone
  │                              │
  │ FILE_OFFER                   │
  │─────────────────────────────►│
  │                              │
  │◄──────── FILE_ACCEPT ────────│
  │                              │
  │════ File Stream ════════════►│
  │════ File Stream ════════════►│
  │                              │
  │◄────── FILE_COMPLETE ────────│
```

---

# 157. Mouse Control Example

```text
Phone                         Laptop
  │                              │
  │ MOUSE_MOVE                   │
  │ dx=12 dy=-3                  │
  │─────────────────────────────►│
  │                              │
  │ MOUSE_BUTTON                 │
  │ LEFT DOWN                    │
  │─────────────────────────────►│
```

---

# 158. Media Control Example

```text
Phone                         Laptop
  │                              │
  │ PLAY                         │
  │─────────────────────────────►│
  │                              │
  │◄──── PLAYBACK_CHANGED ───────│
```

---

# 159. Window Control Example

```text
Phone                         Laptop
  │                              │
  │ WINDOW_LIST                  │
  │─────────────────────────────►│
  │                              │
  │◄──────── WINDOW_LIST ────────│
  │                              │
  │ WINDOW_FOCUS                 │
  │ window_id=42                 │
  │─────────────────────────────►│
```

---

# 160. Complete Connection Flow

```text
                 Device A
                     │
                     │ Discovery
                     ▼
                 Device B
                     │
                     │ QUIC
                     ▼
                  CONNECT
                     │
                   HELLO
                     │
              HELLO_RESPONSE
                     │
             [PAIR flow if unpaired]
                     │
                CAPABILITIES
                     │
          CAPABILITIES_RESPONSE
                     │
                PERMISSIONS
                     │
          PERMISSIONS_RESPONSE
                     │
               SESSION_READY
                     │
          ┌──────────┼──────────┐
          │          │          │
      Clipboard    Files      Input
          │          │          │
          └──────────┼──────────┘
                     │
               Other Services
                     │
              ┌──────┴──────┐
              │             │
            Audio         Video
              │             │
        QUIC Stream    QUIC Stream
```

---

# 161. Wire Format Summary

```text
Remote Protocol v1

Discovery
    UDP multicast
    IPv4: 239.255.42.21:48621
    IPv6: [ff02::524d:5431]:48621
    Datagram: "RMT1" magic + packet type u8 + CBOR payload

Connection
    QUIC / UDP
    Default port: 48622
    ALPN: remote/1
    TLS: 1.3

Serialization
    CBOR

Control framing
    u32 Big Endian
    CBOR payload

Identity
    UUIDv4
    Ed25519

Session
    UUIDv4

Files
    SHA-256

Audio
    Opus

Video
    H.264
```

---

# 162. Service Registry

```text
0x0100 CLIPBOARD
0x0200 FILE_TRANSFER
0x0300 INPUT
0x0400 WINDOW
0x0500 MEDIA
0x0600 AUDIO
0x0700 VIDEO
0x0800 CAMERA
0x0900 MICROPHONE
0x0A00 NOTIFICATIONS
0x0B00 DEVICE
0x0C00 SHARING
```

---

# 163. Capability Registry

```text
0x0001 CLIPBOARD
0x0002 FILE_TRANSFER
0x0003 MOUSE
0x0004 KEYBOARD
0x0005 WINDOW_CONTROL
0x0006 MEDIA_CONTROL
0x0007 AUDIO
0x0008 VIDEO
0x0009 CAMERA
0x000A MICROPHONE
0x000B NOTIFICATIONS
0x000C DEVICE_INFO
0x000D DEVICE_POWER
0x000E SHARING
```

---

# 164. Protocol Invariants

The following rules are mandatory:

1. A session cannot become established without mutual certificate verification (pinning for paired peers, SAS-confirmed pairing otherwise).
2. A peer whose certificate is not pinned is confined to the pairing flow and cannot use privileged services.
3. A service cannot start without mutual capability support.
4. A service cannot start without required permission.
5. File transfers must verify SHA-256.
6. Control messages must use length-prefixed CBOR.
7. Large data must use dedicated streams.
8. A service failure must not terminate unrelated services.
9. Session IDs must never be reused.
10. Pairing nonces must never be reused.
11. Private keys must never leave their device.
12. Discovery information must never be trusted as authentication.
13. Clipboard update IDs must prevent synchronization loops.
14. Unknown optional services must not terminate a session.
15. Platform-specific APIs must remain outside the protocol layer.

---

# 165. Compatibility

A Remote implementation is Protocol v1 compatible when it implements:

```text
Discovery
QUIC
remote/1
CBOR framing
HELLO
HELLO_RESPONSE
PAIR_REQUEST
PAIR_CHALLENGE
PAIR_RESPONSE
PAIR_ACCEPT
PAIR_REJECT
CAPABILITIES
CAPABILITIES_RESPONSE
PERMISSIONS
PERMISSIONS_RESPONSE
SESSION_READY
SESSION_CLOSE
PING
PONG
SERVICE_START
SERVICE_ACCEPT
SERVICE_REJECT
SERVICE_STOP
ERROR
```

and correctly implements the service capabilities it advertises.

---

# 166. Required Core Services

Protocol v1 requires support for:

```text
CORE
CLIPBOARD
FILE_TRANSFER
```

The following optional services are supported by Protocol v1:

```text
INPUT
MEDIA
SHARING
WINDOW
AUDIO
VIDEO
CAMERA
MICROPHONE
NOTIFICATIONS
DEVICE
```

An implementation must advertise the optional services it supports.

A Commands service (remote command execution) is deliberately out of Protocol v1; service ID `0x0D00` is reserved for it in v2.

---

# 167. Protocol Source of Truth

This document defines the Remote Protocol v1 wire contract.

The implementation must conform to this document.

Implementation details may vary between:

```text
Linux
Windows
Android
```

but the wire protocol must remain identical.

---

# 168. Final Architecture

```text
                              REMOTE
                                │
              ┌─────────────────┴─────────────────┐
              │                                   │
          Discovery                            Transport
              │                                   │
       ┌──────┴──────┐                          QUIC
       │             │                           │
      IPv4          IPv6                         │
       │             │                           │
       └──────┬──────┘                           │
              │                                  │
        UDP 48621                          UDP 48622
                                         │
                              ┌──────────┴──────────┐
                              │                     │
                         Control Stream         Data Streams
                              │                     │
                         CBOR Messages        ┌─────┼─────┐
                                              │     │     │
                                            Files Audio Video
                                              │     │     │
                                              │   Camera Microphone
                                              │
                              ┌───────────────┴──────────────┐
                              │                              │
                         Remote Core                  Platform Adapters
                              │                              │
                     ┌────────┼────────┐            ┌────────┼────────┐
                     │        │        │            │        │        │
                  Session  Services  Security     Linux   Windows  Android
                                                    │
                                             ┌──────┴──────┐
                                             │             │
                                          Wayland         X11
```

---

# 169. End State

Remote Protocol v1 defines:

* Device discovery.
* Device identity.
* Pairing.
* Authentication.
* Session establishment.
* Capability negotiation.
* Permission negotiation.
* Control messages.
* Message framing.
* Service negotiation.
* Stream negotiation.
* Clipboard synchronization.
* File transfer.
* Mouse control.
* Keyboard control.
* Window control.
* Media control.
* Sharing.
* Audio streaming.
* Video streaming.
* Camera streaming.
* Microphone streaming.
* Notifications.
* Device information.
* Device power control.
* Error handling.
* Rate limiting.
* Resource limits.
* Security requirements.
* Transport abstraction.
* Linux integration boundaries.
* Windows integration boundaries.
* Android integration boundaries.
* CLI/GUI integration boundaries.
* Protocol compatibility rules.

Deliberately deferred to Protocol v2:

* Bluetooth transport.
* Commands service (remote command execution).
* Clipboard history.
* Adaptive stream renegotiation.
* Rotating discovery identifiers.

Remote Protocol v1 is a Draft. See "Path to Stable" at the top of this document.
