//! Compatibility layer for running Linux binaries via relibc.
//!
//! This module provides stubs for Linux syscalls and ABI shims to allow
//! POSIX-compatible binaries compiled with relibc to run on LucAstra.
//!
//! Features:
//! - `relibc`: Enable relibc compatibility layer (experimental)

pub mod syscall;
pub mod loader;

pub use syscall::SyscallHandler;

use lucastra_core::Result;

/// Initialize the compatibility layer.
pub fn init() -> Result<()> {
    tracing::info!("Initializing LucAstra compatibility layer");
    Ok(())
}

/// Check if relibc feature is enabled.
pub fn is_relibc_enabled() -> bool {
    cfg!(feature = "relibc")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syscall_handler_open_close() {
        let mut handler = SyscallHandler::new();
        
        // Test open
        let fd = handler.handle_syscall(2, &[0x1000, 0, 0]).unwrap();
        assert!(fd > 0);

        // Test close
        let result = handler.handle_syscall(3, &[fd as u64]).unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn test_syscall_handler_dup() {
        let mut handler = SyscallHandler::new();

        // Open file
        let fd1 = handler.handle_syscall(2, &[0x1000, 0, 0]).unwrap();

        // Duplicate fd
        let fd2 = handler.handle_syscall(32, &[fd1 as u64]).unwrap();
        assert_ne!(fd2, fd1);

        // Close both
        handler.handle_syscall(3, &[fd1 as u64]).unwrap();
        handler.handle_syscall(3, &[fd2 as u64]).unwrap();
    }

    #[test]
    fn test_elf_loader_validation() {
        use crate::loader::ElfLoader;

        // Valid ELF header
        let valid_elf = b"\x7fELF\x02\x01\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00";
        assert!(ElfLoader::validate_elf(valid_elf));

        // Invalid header
        let invalid = b"INVALID";
        assert!(!ElfLoader::validate_elf(invalid));
    }

    #[test]
    fn test_fat32_boot_sector_parsing() {
        use crate::loader::FAT32Reader;

        // Create a minimal valid FAT32 boot sector (512 bytes)
        let mut boot_sector = vec![0u8; 512];
        boot_sector[0] = 0xEB;  // jump_boot
        boot_sector[1] = 0x3C;
        boot_sector[2] = 0x90;
        boot_sector[11] = 0x00; // bytes_per_sector (little endian)
        boot_sector[12] = 0x04; // 1024 bytes
        boot_sector[510] = 0x55; // signature
        boot_sector[511] = 0xAA;

        let mut reader = FAT32Reader::new();
        let result = reader.parse_boot_sector(&boot_sector);
        assert!(result.is_ok());
        assert!(reader.boot_sector().is_some());
    }
}
