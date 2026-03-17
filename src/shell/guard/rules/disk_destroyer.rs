// src/shell/guard/rules/disk_destroyer.rs

use super::destructive_fs::token_args;
use crate::parser::ast::Command;
use crate::shell::guard::Guard;
use crate::shell::guard::error::GuardError;

/// Dispositivos de bloque principales — escritura directa = destrucción
const BLOCK_DEVICES: &[&str] = &[
    "/dev/sda",
    "/dev/sdb",
    "/dev/sdc",
    "/dev/sdd",
    "/dev/nvme0n1",
    "/dev/nvme1n1",
    "/dev/nvme2n1",
    "/dev/vda",
    "/dev/vdb",
    "/dev/xvda",
    "/dev/xvdb",
    "/dev/mmcblk0",
];

// ══════════════════════════════════════════════════════════════
// DdGuard — El Triturador
// ══════════════════════════════════════════════════════════════

pub struct DdGuard;

impl DdGuard {
    pub fn new() -> Self {
        Self
    }
}

impl Guard for DdGuard {
    fn check_command(&self, cmd: &Command) -> Result<(), GuardError> {
        if cmd.name != "dd" {
            return Ok(());
        }

        let args = token_args(&cmd.args);

        for arg in &args {
            // Detecta of=/dev/sda*, of=/dev/nvme* etc.
            if let Some(target) = arg.strip_prefix("of=") {
                if BLOCK_DEVICES
                    .iter()
                    .any(|&dev| target == dev || target.starts_with(dev))
                {
                    return Err(GuardError::DiskDestroyer {
                        reason: format!(
                            "dd writing directly to block device '{target}' \
                             would destroy all data on that disk"
                        ),
                    });
                }
            }
        }
        Ok(())
    }
}

// ══════════════════════════════════════════════════════════════
// MkfsGuard — El Formateador
// ══════════════════════════════════════════════════════════════

const MKFS_CONFIRMATION_FLAG: &str = "--yes-i-know-what-i-am-doing";

pub struct MkfsGuard;

impl MkfsGuard {
    pub fn new() -> Self {
        Self
    }
}

impl Guard for MkfsGuard {
    fn check_command(&self, cmd: &Command) -> Result<(), GuardError> {
        // Detecta mkfs, mkfs.ext4, mkfs.btrfs, etc.
        if !cmd.name.starts_with("mkfs") {
            return Ok(());
        }

        let args = token_args(&cmd.args);

        // Permite si tiene el flag de confirmación explícita
        if args.iter().any(|a| a == MKFS_CONFIRMATION_FLAG) {
            return Ok(());
        }

        Err(GuardError::RequiresConfirmation {
            reason: format!(
                "'{}' will FORMAT a filesystem. \
                 Pass '{}' to confirm you know what you're doing.",
                cmd.name, MKFS_CONFIRMATION_FLAG
            ),
        })
    }
}
