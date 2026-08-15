# Remote — Architecture

## 1. Purpose

This document defines the high-level architecture of Remote.

The purpose of the architecture is to establish clear boundaries between the Remote core, communication protocol, network transport, platform integrations, and user-facing applications.

The architecture should allow Remote to evolve without requiring unrelated components to be rewritten whenever a new service, platform, or transport is introduced.

---

# 2. Architectural Goals

The Remote architecture shall prioritize:

- Modularity.
- Separation of concerns.
- Platform independence.
- Service isolation.
- Transport independence.
- Protocol extensibility.
- Testability.
- Low coupling.
- Reusability.
- Long-term maintainability.

The architecture should make it possible to add functionality without creating dependencies between unrelated services.

---

# 3. High-Level Architecture

Remote is divided into several conceptual layers:

```text
┌─────────────────────────────────────────────┐
│                User Interfaces              │
│                                             │
│              CLI / GUI / Other              │
└──────────────────────┬──────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────┐
│              Remote Application             │
│                                             │
│       Device management / lifecycle         │
└──────────────────────┬──────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────┐
│                   Services                  │
│                                             │
│ Clipboard · Files · Audio · Video · Input   │
│ Camera · Microphone · Media · Notifications │
└──────────────────────┬──────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────┐
│                  Protocol                   │
│                                             │
│ Packets · Messages · Handshake · Capabilities│
└──────────────────────┬──────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────┐
│                  Transport                  │
│                                             │
│       Reliable / streaming communication    │
└──────────────────────┬──────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────┐
│                Platform / OS                │
│                                             │
│ Linux · Windows · Android                   │
└─────────────────────────────────────────────┘
```

The exact implementation of each layer may evolve.

---

# 4. Core Architectural Principle

Remote should be **service-oriented rather than feature-monolithic**.

A feature such as clipboard synchronization should not contain its own networking implementation.

Instead:

```text
Clipboard Service
       │
       ▼
Remote Protocol
       │
       ▼
Transport
       │
       ▼
Network
```

The clipboard service should only be responsible for clipboard-related behavior.

It should not need to know whether communication occurs through Wi-Fi, Bluetooth, or another transport.

---

# 5. Components

The current project consists of three primary Rust crates:

```text
remote        (daemon, binary `remoted`)
remote-cli    (CLI, binary `remote`)
remote-core   (shared library)
```

These crates should have clearly separated responsibilities.

---

# 6. `remote-core`

`remote-core` is the foundational library of Remote.

It should contain functionality that can be shared by different Remote applications.

The core should not depend on a graphical interface.

It should provide the building blocks required to:

- Represent Remote devices.
- Discover devices.
- Establish connections.
- Handle the Remote protocol.
- Negotiate capabilities.
- Manage services.
- Handle communication.
- Manage device state.
- Expose platform-independent interfaces.

Conceptually:

```text
remote-core
│
├── discovery
├── identity
├── pairing
├── connection
├── protocol
├── transport
├── capabilities
├── services
└── events
```

The exact module structure may evolve during implementation.

---

# 7. `remote`

`remote` is the primary Remote application (binary `remoted`, following the Unix daemon convention).

It should provide the long-running process responsible for operating Remote on a computer.

Conceptually, it acts as the Remote daemon.

Responsibilities may include:

- Starting Remote.
- Loading configuration.
- Initializing the core.
- Managing device connections.
- Running services.
- Handling background operation.
- Exposing functionality to user interfaces.
- Managing platform integrations.

The daemon should be usable without a graphical interface.

Example:

```text
Remote Daemon (remoted)
     │
     ├── remote-core
     │
     ├── Services
     │
     └── Platform Integration
```

---

# 8. `remote-cli`

`remote-cli` provides the command-line interface for Remote (binary `remote`).

The CLI should interact with the Remote core rather than implementing Remote functionality itself.

Examples of commands may eventually include:

```text
remote devices
remote pair
remote connect
remote disconnect
remote status
remote clipboard
remote send
remote media
remote input
```

The CLI should remain lightweight.

It should not contain duplicate implementations of services, networking, or protocol logic.

---

# 9. User Interface Separation

Remote should separate user interfaces from the underlying system.

Conceptually:

```text
             ┌─────────────┐
             │     GUI     │
             └──────┬──────┘
                    │
             ┌──────▼──────┐
             │     CLI     │
             └──────┬──────┘
                    │
                    ▼
             ┌─────────────┐
             │ Remote Core │
             └─────────────┘
```

The CLI and GUI are interfaces to Remote.

They are not the Remote system itself.

This allows other interfaces to be developed later without duplicating functionality.

---

# 10. Service Layer

Remote functionality should be divided into independent services.

Initial services include:

```text
Clipboard
Files
Input
Windows
Audio
Video
Camera
Microphone
Media
Notifications
Device Status
Sharing
```

A Commands service (remote command execution) is deferred to Protocol v2.

Each service should have:

- A clear responsibility.
- Defined inputs and outputs.
- Defined permissions.
- Protocol messages.
- Platform-specific implementations where necessary.

---

# 11. Service Isolation

Services should be isolated from one another.

For example:

```text
Video Service
     ✕
Clipboard Service
```

The video service should not directly control the clipboard service.

Communication between components should occur through defined interfaces or events.

A failure in one service should not unnecessarily terminate unrelated services.

Example:

```text
Video stream fails

        ↓

Video Service stops

        ↓

Clipboard continues
Audio continues
Device connection continues
```

---

# 12. Platform Abstraction

Remote must support multiple operating systems and desktop environments.

Platform-specific behavior should therefore be isolated.

Conceptually:

```text
                 Remote Service
                       │
                       ▼
               Platform Interface
                       │
            ┌──────────┼──────────┐
            ▼          ▼          ▼
          Linux      Windows    Android
```

For Linux:

```text
Linux
 ├── Wayland
 ├── X11
 └── Other platform APIs
```

The core protocol should not contain Linux-specific or Windows-specific behavior.

---

# 13. Platform Integration

Some Remote services necessarily depend on operating-system functionality.

Examples include:

- Mouse injection.
- Keyboard injection.
- Window management.
- Camera exposure.
- Microphone access.
- Audio device management.
- Webcam creation.
- Clipboard access.
- Notifications.

These should be implemented through platform-specific adapters.

Example:

```text
Input Service
     │
     ▼
Input Platform Interface
     │
     ├── Linux
     │    ├── Wayland
     │    └── X11
     │
     └── Windows
```

This prevents platform-specific implementation details from leaking into the protocol.

---

# 14. Protocol Layer

The protocol defines how Remote devices communicate.

It is responsible for:

- Message representation.
- Packet structure.
- Message types.
- Protocol versions.
- Handshakes.
- Capability negotiation.
- Service communication.
- Error representation.
- Session information.

The protocol should remain independent from the user interface.

---

# 15. Transport Layer

The transport layer is responsible for moving protocol data between devices.

Services should not directly interact with sockets.

Conceptually:

```text
Service
   ↓
Protocol
   ↓
Transport
   ↓
Network
```

This allows the same service to operate over different transports.

Potential transports include:

```text
Wi-Fi (v1)
Bluetooth (future)
Other future transports
```

The initial implementation should not assume that one transport will always be sufficient.

---

# 16. Control Plane and Data Plane

Remote should distinguish between control communication and high-bandwidth data communication.

### Control Plane

Used for:

- Discovery.
- Pairing.
- Authentication.
- Capability negotiation.
- Service control.
- Device status.
- Commands.
- Metadata.

### Data Plane

Used for:

- Files.
- Audio.
- Video.
- Camera streams.
- Microphone streams.
- Other high-bandwidth content.

Conceptually:

```text
                  Remote Connection
                         │
             ┌───────────┴───────────┐
             │                       │
             ▼                       ▼
       Control Plane            Data Plane
             │                       │
       Handshake                Video
       Capabilities             Audio
       Commands                 Files
       Metadata                 Camera
```

The architecture should allow these paths to use different communication characteristics when appropriate.

---

# 17. Event System

Remote should use an internal event system for communication between loosely coupled components.

Examples of events include:

```text
DeviceDiscovered
DeviceConnected
DeviceDisconnected

ClipboardChanged

FileTransferStarted
FileTransferProgress
FileTransferCompleted

AudioStarted
AudioStopped

VideoStarted
VideoStopped

PermissionChanged

CapabilityChanged
```

Events should allow components to react to state changes without directly depending on one another.

---

# 18. Device Model

A Remote device should be represented by a common device model.

A device may contain information such as:

```text
Device ID
Device Name
Platform
Remote Version
Connection State
Capabilities
Permissions
```

The device model should not assume that a device is always a computer or always a phone.

A Remote device is any supported endpoint capable of participating in the Remote protocol.

---

# 19. Capability Model

Remote should represent device functionality as capabilities.

Example:

```text
Device A

Capabilities:
    clipboard
    file-transfer
    mouse
    keyboard
    audio-output
```

Another device may advertise:

```text
Device B

Capabilities:
    clipboard
    file-transfer
    microphone
    camera
```

The Remote system determines which interactions are possible based on the capabilities exposed by both devices.

---

# 20. Capability Negotiation

When two devices establish an authorized session, they should negotiate their supported capabilities.

Conceptually:

```text
Device A                    Device B
   │                           │
   │ capabilities ──────────► │
   │                           │
   │ ◄──────── capabilities    │
   │                           │
   │                           │
   │   Common capabilities     │
   │          ↓                │
   │      Start services       │
```

Capability negotiation should support versioning.

Example:

```text
clipboard: 2
audio: 1
video: 1
```

A device should not need to support every available capability in order to establish a connection.

---

# 21. Connection Lifecycle

A typical Remote connection should follow a lifecycle similar to:

```text
Discovery
    ↓
Identification
    ↓
Pairing
    ↓
Authentication
    ↓
Capability Negotiation
    ↓
Session Established
    ↓
Services Active
    ↓
Disconnect
```

The protocol should define each stage explicitly.

---

# 22. Pairing and Trust

Pairing establishes trust between devices by pinning the peer's TLS certificate after the user confirms a short verification code (SAS) on both devices.

Once paired, a device reconnects without manual approval: each session is mutually authenticated via TLS with certificate pinning (details in the protocol document).

Users should be able to:

- View paired devices.
- Revoke a device.
- Re-pair a device.
- Rename a device.
- Modify permissions.

---

# 23. Permissions

Permissions should be independent from device pairing.

Pairing establishes that:

```text
"This device is trusted."
```

Permissions determine:

```text
"What is this device allowed to do?"
```

Example:

```text
Trusted Device
       │
       ├── Clipboard ✓
       ├── Files ✓
       ├── Media ✓
       ├── Mouse ✓
       ├── Keyboard ✗
       ├── Camera ✗
       └── Microphone ✗
```

This distinction should be preserved throughout the architecture.

---

# 24. Security Boundary

Security-sensitive operations must pass through explicit authorization boundaries.

Examples:

```text
Camera
Microphone
Keyboard
Mouse
Command Execution
Window Control
File Access
```

A service must not assume that because a device is connected, it has permission to perform privileged operations.

---

# 25. Configuration

Configuration should be separated from runtime state.

Configuration may include:

```text
Device identity
Paired devices
Permissions
Enabled services
Transport configuration
Logging configuration
Service preferences
```

Runtime state may include:

```text
Active connections
Current transfers
Current streams
Current capabilities
Temporary session data
```

Runtime state should not unnecessarily be persisted.

---

# 26. Error Handling

Errors should be represented at the appropriate layer.

Examples:

```text
Network Error
     ↓
Transport Error
     ↓
Protocol Error
     ↓
Service Error
```

A lower-level failure should be translated into a meaningful error for the layer above it.

Services should not need to understand operating-system-specific socket errors.

---

# 27. Logging and Diagnostics

Remote should provide structured logging.

Logging should be useful for:

- Connection failures.
- Pairing failures.
- Protocol errors.
- Service failures.
- Platform integration problems.
- Performance diagnostics.

The CLI should provide a way to inspect relevant diagnostic information.

---

# 28. Testing Architecture

The architecture should allow individual components to be tested independently.

Tests should eventually exist for:

```text
Protocol
Transport
Discovery
Pairing
Capability Negotiation
Services
Platform Adapters
CLI
```

Where practical, services should be testable without requiring a physical mobile device or real network connection.

---

# 29. Current Rust Workspace

The current workspace is:

```text
remote/
│
├── remote/
│   └── src/
│       └── main.rs
│
├── remote-cli/
│   └── src/
│       └── main.rs
│
├── remote-core/
│   └── src/
│       └── lib.rs
│
├── Cargo.toml
└── Cargo.lock
```

This structure provides a good initial foundation for the architecture.

The intended responsibilities are:

```text
remote
    Application / daemon (binary `remoted`)

remote-cli
    Command-line interface (binary `remote`)

remote-core
    Shared Remote functionality
```

---

# 30. Proposed Core Organization

As the project grows, `remote-core` should evolve toward clearly separated modules.

A possible organization is:

```text
remote-core/
└── src/
    ├── lib.rs
    │
    ├── discovery/
    │   └── mod.rs
    │
    ├── identity/
    │   └── mod.rs
    │
    ├── pairing/
    │   └── mod.rs
    │
    ├── connection/
    │   └── mod.rs
    │
    ├── protocol/
    │   ├── mod.rs
    │   ├── packet.rs
    │   ├── header.rs
    │   ├── handshake.rs
    │   └── capabilities.rs
    │
    ├── transport/
    │   └── mod.rs
    │
    ├── services/
    │   ├── mod.rs
    │   ├── clipboard.rs
    │   ├── files.rs
    │   ├── input.rs
    │   ├── audio.rs
    │   ├── video.rs
    │   ├── camera.rs
    │   └── microphone.rs
    │
    ├── events/
    │   └── mod.rs
    │
    └── error.rs
```

This is an architectural direction rather than a strict requirement.

The actual structure should be adjusted as implementation reveals better boundaries.

---

# 31. Dependency Direction

Dependencies should primarily flow toward the core abstractions.

Conceptually:

```text
CLI
 │
 ▼
Application
 │
 ▼
Core
 │
 ├── Protocol
 ├── Services
 ├── Transport
 └── Platform Interfaces
```

A service should not depend on the CLI.

The protocol should not depend on the GUI.

The transport should not depend on a specific service.

Platform implementations should implement defined interfaces rather than forcing platform-specific concepts into the core.

---

# 32. Dependency Rules

The following rules should generally apply:

### Rule 1

User interfaces may depend on Remote core functionality.

### Rule 2

Services may depend on protocol and abstract platform interfaces.

### Rule 3

Services should not depend on user interfaces.

### Rule 4

The protocol should not depend on services that implement it.

### Rule 5

Transport should not depend on application-level services.

### Rule 6

Platform-specific code should remain isolated.

### Rule 7

Core abstractions should avoid unnecessary platform-specific types.

---

# 33. Extensibility

The architecture should allow new functionality to be added without modifying unrelated components.

For example, adding a new service:

```text
New Service
    ↓
Service Interface
    ↓
Protocol Messages
    ↓
Existing Transport
```

should not require rewriting:

```text
Clipboard
Audio
Video
CLI
Discovery
```

---

# 34. Service Lifecycle

Services should have an explicit lifecycle.

A conceptual lifecycle is:

```text
Unavailable
    ↓
Available
    ↓
Starting
    ↓
Running
    ↓
Stopping
    ↓
Stopped
```

A service may also enter an error state.

```text
Running
   ↓
Error
   ↓
Recovery
   ↓
Running
```

The exact lifecycle implementation will be defined later.

---

# 35. Data Flow

A typical service operation should follow this general path:

```text
User / Platform
      │
      ▼
Service
      │
      ▼
Protocol Message
      │
      ▼
Transport
      │
      ▼
Network
      │
      ▼
Remote Transport
      │
      ▼
Remote Protocol
      │
      ▼
Remote Service
      │
      ▼
Remote Platform
```

This separation allows the same service architecture to operate across different platforms.

---

# 36. High-Bandwidth Data

Services such as audio, video, camera, and file transfer may generate significantly more data than control services.

The architecture should therefore avoid forcing all data through a single generic communication mechanism when doing so would negatively affect:

- Latency.
- Throughput.
- Reliability.
- Resource usage.

The protocol should provide a mechanism for services to establish appropriate data channels when necessary.

---

# 37. Concurrency

Remote is expected to perform multiple operations simultaneously.

Examples:

```text
Clipboard synchronization
        +
File transfer
        +
Audio stream
        +
Device status
        +
Mouse input
```

The architecture should therefore support concurrent services without requiring them to block one another.

---

# 38. Resource Management

Long-running services should explicitly manage:

- Connections.
- Streams.
- Tasks.
- Buffers.
- Files.
- Platform resources.

Resources should be released when services stop or connections terminate.

---

# 39. Architecture Evolution

This architecture is intentionally not considered final.

As implementation progresses, architectural decisions may change.

Major architectural changes should be documented through RFCs or architecture decision records.

The goal is not to predict every future requirement, but to establish boundaries that allow Remote to evolve safely.

---

# 40. Summary

Remote is organized around a modular core, independent services, an explicit communication protocol, abstract transports, and platform-specific integrations.

The intended relationship is:

```text
                    Remote
                       │
          ┌────────────┼────────────┐
          │            │            │
         CLI          GUI        Other UI
          │            │            │
          └────────────┼────────────┘
                       │
                       ▼
                  Remote Core
                       │
        ┌──────────────┼──────────────┐
        │              │              │
    Services        Protocol       Events
        │              │              │
        └──────────────┼──────────────┘
                       │
                    Transport
                       │
              ┌────────┴────────┐
              │                 │
             Wi-Fi         Bluetooth
             (v1)           (future)
              │                 │
              └────────┬────────┘
                       │
                    Devices
```

The architecture intentionally separates **what Remote does** from **how devices communicate** and from **how each operating system provides the required functionality**.

This separation is the foundation for Remote's long-term cross-platform and service-oriented design.
