<div align="center">
<br/>

# self-healing

### Autonomous fault detection, diagnosis, and recovery for Sorabbyngo OS.

<br/>

[![Division](https://img.shields.io/badge/Division-Sorabbyngo_OS-6C3FC4?style=for-the-badge)](https://github.com/sorabbyngo-os)
[![Lang](https://img.shields.io/badge/Lang-Rust-AA66FF?style=for-the-badge)](https://github.com/sorabbyngo-os/self-healing)
[![Status](https://img.shields.io/badge/Status-In_Development-FF6B35?style=for-the-badge)](https://github.com/sorabbyngo-os/self-healing)

</div>

---

## What It Does

self-healing monitors the health of all Sorabbyngo OS components and autonomously recovers from failures — service crashes, resource exhaustion, network partitions, and hardware faults — without requiring human intervention.

Designed for edge deployments in Africa where manual intervention is not possible for hours or days.

---

## Features

| Feature | Description |
|---|---|
| **Service Watchdog** | Monitors all systemd services — restarts on crash with backoff |
| **Resource Guardian** | Detects and responds to memory/CPU exhaustion |
| **Network Recovery** | Automatic reconnection for Byngonet mesh and uplink |
| **Disk Health** | Filesystem integrity checks, bad block detection |
| **Rollback Engine** | Boots to last-known-good on repeated failure |
| **Byngox Agent** | AI-assisted fault diagnosis via Byngox reasoning |
| **Telemetry** | All recovery actions reported to sorabyte-monitor |

---

## Architecture

```
self-healing/
├── src/
│   ├── watchdog/     # Service health monitoring and restart
│   ├── resources/    # CPU/memory pressure detection and response
│   ├── network/      # Byngonet and uplink recovery
│   ├── disk/         # Filesystem health and integrity
│   ├── rollback/     # Last-known-good boot management
│   ├── diagnosis/    # AI-assisted fault diagnosis (Byngox)
│   └── telemetry/    # Recovery event reporting
├── Cargo.toml
└── README.md
```

---

## Recovery Philosophy

**Try fast, try often, escalate slowly.** For a service crash: restart immediately, then with 5s backoff, then 30s, then attempt diagnosis, then rollback if unresolved. Never sit in a failed state when a recovery action is available.

---

## Stack

- **Language:** Rust
- **Integration:** systemd D-Bus, procfs, sysfs
- **AI diagnosis:** Byngox agent (sorabbyngo-os/byngox)
- **Telemetry:** sorabyte-monitor
- **OS target:** All Sorabbyngo OS surfaces

---

*Sorabbyngo OS · Kisii, Kenya · 2026*
