//! self-healing/src/snapshot_bridge.rs
//!
//! Wires the self-healing engine to sorabbyngofs snapshots.
//! Add this module to the existing self-healing repo.
//!
//! Behaviour:
//!   - Before every package install: create a named snapshot
//!   - On boot failure detection:    roll back to last known good snapshot
//!   - On kernel panic / watchdog:   trigger recovery partition boot
//!   - On filesystem corruption:     invoke sorabbyngofs self-healing journal

use std::fs;
use std::io::Write;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// Snapshot operations — talk to sorabbyngofs via its ioctl interface
/// or via the sorabbyngofs CLI tool (sorabbyngofs-ctl)
pub struct SnapshotBridge {
    mountpoint: String,
}

impl SnapshotBridge {
    pub fn new(mountpoint: &str) -> Self {
        Self { mountpoint: mountpoint.to_string() }
    }

    /// Create a pre-install snapshot. Called by package manager hook.
    pub fn pre_install_snapshot(&self, pkg_name: &str) -> Result<String, String> {
        let ts   = epoch_secs();
        let name = format!("pre-install-{}-{}", pkg_name, ts);

        let out = Command::new("sorabbyngofs-ctl")
            .args(["snapshot", "create", "--name", &name, &self.mountpoint])
            .output()
            .map_err(|e| format!("sorabbyngofs-ctl error: {}", e))?;

        if out.status.success() {
            self.audit(&format!("snapshot created: {}", name));
            Ok(name)
        } else {
            Err(String::from_utf8_lossy(&out.stderr).to_string())
        }
    }

    /// Roll back to the most recent snapshot. Called on boot failure.
    pub fn rollback_latest(&self) -> Result<(), String> {
        let out = Command::new("sorabbyngofs-ctl")
            .args(["snapshot", "rollback", "--latest", &self.mountpoint])
            .output()
            .map_err(|e| format!("rollback error: {}", e))?;

        if out.status.success() {
            self.audit("rollback: rolled back to latest snapshot");
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&out.stderr).to_string())
        }
    }

    /// Roll back to a specific named snapshot.
    pub fn rollback_to(&self, snapshot_name: &str) -> Result<(), String> {
        let out = Command::new("sorabbyngofs-ctl")
            .args(["snapshot", "rollback",
                   "--name", snapshot_name,
                   &self.mountpoint])
            .output()
            .map_err(|e| format!("rollback error: {}", e))?;

        if out.status.success() {
            self.audit(&format!("rollback: -> {}", snapshot_name));
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&out.stderr).to_string())
        }
    }

    /// Trigger recovery boot (sets next-boot flag, then reboots).
    /// Called when rollback alone cannot fix the issue.
    pub fn trigger_recovery_boot(&self) -> Result<(), String> {
        self.audit("RECOVERY: triggering recovery partition boot");

        // Write recovery flag to EFI variable or grub env
        let _ = Command::new("grub-editenv")
            .args(["/boot/grub/grubenv", "set", "sorabbyngo_recovery=1"])
            .output();

        // Reboot into recovery
        Command::new("systemctl")
            .args(["reboot", "--firmware-setup"])
            .output()
            .map_err(|e| format!("reboot error: {}", e))?;

        Ok(())
    }

    /// Invoke sorabbyngofs journal replay on filesystem corruption.
    pub fn heal_journal(&self) -> Result<(), String> {
        let out = Command::new("sorabbyngofs-ctl")
            .args(["journal", "replay", &self.mountpoint])
            .output()
            .map_err(|e| format!("journal replay error: {}", e))?;

        if out.status.success() {
            self.audit("journal replay: complete");
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&out.stderr).to_string())
        }
    }

    fn audit(&self, msg: &str) {
        let ts   = epoch_secs();
        let line = format!("[{}] {}\n", ts, msg);
        let _ = fs::OpenOptions::new()
            .create(true).append(true)
            .open("/var/log/sorabbyngo/self_healing.log")
            .and_then(|mut f| f.write_all(line.as_bytes()));
    }
}

fn epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

// ── Package manager hook (called by sorabbyngo package manager) ────────────

/// Drop-in hook for the package manager.
/// Call before_install() before any install, after_install_fail() on failure.
pub struct PkgHook {
    bridge:           SnapshotBridge,
    last_snapshot:    Option<String>,
}

impl PkgHook {
    pub fn new() -> Self {
        Self {
            bridge:        SnapshotBridge::new("/"),
            last_snapshot: None,
        }
    }

    pub fn before_install(&mut self, pkg: &str) {
        match self.bridge.pre_install_snapshot(pkg) {
            Ok(name) => {
                println!("[self-healing] snapshot created: {}", name);
                self.last_snapshot = Some(name);
            }
            Err(e) => eprintln!("[self-healing] snapshot failed: {}", e),
        }
    }

    pub fn after_install_fail(&self, pkg: &str) {
        eprintln!("[self-healing] Install of {} failed — rolling back", pkg);
        match &self.last_snapshot {
            Some(snap) => {
                let _ = self.bridge.rollback_to(snap);
            }
            None => {
                let _ = self.bridge.rollback_latest();
            }
        }
    }
}
