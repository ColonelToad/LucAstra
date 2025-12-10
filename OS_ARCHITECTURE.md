# LucAstra OS Architecture

## Overview
LucAstra is a lightweight OS prototype built entirely in Rust with embedded LLM support. The architecture is modular and device-centric, designed for laptop use with USB, keyboard, and mouse support.

## Core Components

### 1. **Kernel** (`kernel/`)
- Boot coordination and lifecycle management
- Tracing/logging initialization
- Interrupt and event loop stubs
- Service registry initialization

**Key functions:**
- `boot(config)` - Initialize the OS
- `shutdown()` - Graceful shutdown

### 2. **HAL (Hardware Abstraction Layer)** (`hal/`)
Trait-based abstractions for device operations:
- `BlockDevice` - USB drives, disks (read/write sectors)
- `FileSystemDriver` - Filesystem I/O (mount, list, read, write)
- `InputDriver` - Keyboard/mouse input polling

**Mock implementations provided for MVP testing.**

### 3. **Device Manager** (`devices/`)
Service that enumerates and manages hardware:
- Scans for USB block devices
- Detects input devices (keyboard, mouse)
- Tracks mounted/unmounted state

**Future enhancement:** Wire to real libusb/udev on Linux, device drivers on Windows.

### 4. **Filesystem Manager** (`fs/`)
Abstraction layer for mounted filesystems:
- Mounts/unmounts drivers at paths
- Routes file operations to appropriate driver
- Supports multiple concurrent filesystems

**Current:** Mock in-memory implementation. Future: FAT32, ext4 drivers via `BlockDevice` trait.

### 5. **Input Manager** (`input/`)
Handles keyboard and mouse events:
- Polls registered input drivers
- Buffers events in a queue
- Non-blocking event retrieval

### 6. **Compatibility Layer** (`compat/`)
Linux/POSIX compatibility stubs for running Linux binaries:
- Syscall handler (open, read, write, close, ioctl, lseek, dup, exit)
- ELF loader skeleton for binary loading
- Optional `relibc` feature for deeper integration

**Current:** Stubs for MVP. Future: Full relibc integration for LibreOffice, other Linux apps.

### 7. **Core Types** (`core/`)
Shared message contracts and error types:
- `Command` / `CommandPayload` - User/service requests
- `Response` / `ResponsePayload` - Responses and results
- `DeviceInfo`, `InputEvent` - Data structures
- `LuCastraError` - Unified error type

### 8. **App Binary** (`app/`)
Main entry point orchestrating all services:
- Boots kernel
- Initializes device manager, filesystem, input
- Runs a command loop (example handlers for ListDevices, Status, Echo)
- Ready to integrate GUI or advanced LLM pipelines

## Data Flow

```
[Input: CLI, GUI, LLM] 
       ↓
   App/Router
       ↓
 [Core Command]
       ↓
Device Manager ──→ [USB, Keyboard, Mouse enumeration]
Filesystem Manager ──→ [File I/O via mounted drivers]
Input Manager ──→ [Input buffering & polling]
Compat Layer ──→ [Syscall translation for Linux binaries]
       ↓
   [Response]
       ↓
[Output: GUI, Log, Result]
```

## Configuration

- **Environment**: Set `LUC_ASTRA_MODEL_DIR` to point to the 7B model
- **Tracing**: Control log level via `RUST_LOG` (e.g., `RUST_LOG=info cargo run`)
- **Features**: `relibc` feature available but disabled by default for lightweight builds

## Testing

Run the MVP boot sequence:
```bash
cargo run -p lucastra-app
```

Expected output:
- Kernel boot message
- Device scan (3 mock devices: USB, keyboard, mouse)
- Filesystem mounted at /mnt/root
- Command loop executes ListDevices, Status, and Echo examples
- Boot complete message

## MVP Checklist

- [x] Boot kernel and initialize services
- [x] USB device enumeration (mock)
- [x] Input device enumeration (mock)
- [x] Filesystem mounting abstraction
- [x] Filesystem I/O (read/write via drivers)
- [x] Input event buffering
- [x] Syscall stubs for compat layer
- [x] Command routing and response handling
- [ ] Real USB device driver
- [ ] FAT32/ext4 filesystem drivers
- [ ] ELF loader for Linux binaries
- [ ] LLM integration (search, context, response)
- [ ] GUI integration (iced frontend)

## Next Steps

1. **LLM Integration**: Hook LLM responder to command loop; read 7B model from `LUC_ASTRA_MODEL_DIR`.
2. **BM25 Search Service**: Implement in-memory inverted index for filesystem documents.
3. **GUI Integration**: Connect iced frontend to app command handler.
4. **Real Drivers**: Implement FAT32 reader, actual USB enumeration (libusb on Linux, WinUSB on Windows).
5. **Relibc Integration**: Flesh out syscall handler and ELF loader for running unmodified Linux binaries.

---

**Built with:** Rust, iced (GUI), tracing (observability)  
**License:** MIT  
**Repository:** https://github.com/ColonelToad/LucAstra
