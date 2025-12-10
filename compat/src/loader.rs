//! ELF loader and FAT32 minimal reader for running Linux binaries.
//!
//! This module provides minimal ELF parsing, FAT32 reading, and binary loading
//! for relibc-compiled binaries.

use lucastra_core::Result;

/// FAT32 Boot Sector structure (minimal).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FAT32BootSector {
    pub jump_boot: [u8; 3],
    pub oem_name: [u8; 8],
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub reserved_sectors: u16,
    pub num_fats: u8,
    pub root_entries: u16,
    pub total_sectors_16: u16,
    pub media: u8,
    pub sectors_per_fat_16: u16,
    pub sectors_per_track: u16,
    pub heads: u16,
    pub hidden_sectors: u32,
    pub total_sectors_32: u32,
    pub sectors_per_fat_32: u32,
    pub root_cluster: u32,
    pub fsinfo_sector: u16,
}

/// Minimal FAT32 filesystem parser.
pub struct FAT32Reader {
    boot_sector: Option<FAT32BootSector>,
}

impl FAT32Reader {
    pub fn new() -> Self {
        Self {
            boot_sector: None,
        }
    }

    /// Parse a FAT32 boot sector from raw bytes.
    pub fn parse_boot_sector(&mut self, data: &[u8]) -> Result<()> {
        if data.len() < 90 {
            return Err(lucastra_core::LuCastraError::FilesystemError(
                "Boot sector too small".to_string(),
            ));
        }

        // Validate FAT32 signature (0xAA55 at offset 510)
        if data.len() >= 512 {
            let sig = u16::from_le_bytes([data[510], data[511]]);
            if sig != 0xAA55 {
                return Err(lucastra_core::LuCastraError::FilesystemError(
                    "Invalid FAT32 signature".to_string(),
                ));
            }
        }

        let boot_sector = FAT32BootSector {
            jump_boot: [data[0], data[1], data[2]],
            oem_name: [
                data[3], data[4], data[5], data[6], data[7], data[8], data[9], data[10],
            ],
            bytes_per_sector: u16::from_le_bytes([data[11], data[12]]),
            sectors_per_cluster: data[13],
            reserved_sectors: u16::from_le_bytes([data[14], data[15]]),
            num_fats: data[16],
            root_entries: u16::from_le_bytes([data[17], data[18]]),
            total_sectors_16: u16::from_le_bytes([data[19], data[20]]),
            media: data[21],
            sectors_per_fat_16: u16::from_le_bytes([data[22], data[23]]),
            sectors_per_track: u16::from_le_bytes([data[24], data[25]]),
            heads: u16::from_le_bytes([data[26], data[27]]),
            hidden_sectors: u32::from_le_bytes([data[28], data[29], data[30], data[31]]),
            total_sectors_32: u32::from_le_bytes([data[32], data[33], data[34], data[35]]),
            sectors_per_fat_32: u32::from_le_bytes([data[36], data[37], data[38], data[39]]),
            root_cluster: u32::from_le_bytes([data[44], data[45], data[46], data[47]]),
            fsinfo_sector: u16::from_le_bytes([data[48], data[49]]),
        };

        tracing::info!("Parsed FAT32 boot sector: {} bytes/sector, {} sectors/cluster",
            boot_sector.bytes_per_sector, boot_sector.sectors_per_cluster);

        self.boot_sector = Some(boot_sector);
        Ok(())
    }

    /// Get the boot sector if parsed.
    pub fn boot_sector(&self) -> Option<&FAT32BootSector> {
        self.boot_sector.as_ref()
    }

    /// Calculate the LBA of the FAT table.
    pub fn fat_lba(&self) -> Option<u32> {
        self.boot_sector.as_ref().map(|bs| bs.hidden_sectors + bs.reserved_sectors as u32)
    }

    /// Calculate the LBA of the root directory.
    pub fn root_dir_lba(&self) -> Option<u32> {
        self.boot_sector.as_ref().map(|bs| {
            let fat_size = bs.sectors_per_fat_32;
            bs.hidden_sectors + bs.reserved_sectors as u32 + (bs.num_fats as u32 * fat_size)
        })
    }
}

impl Default for FAT32Reader {
    fn default() -> Self {
        Self::new()
    }
}

/// Minimal ELF header validation and parsing.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ELFHeader {
    pub magic: [u8; 4],
    pub class: u8,    // 1=32-bit, 2=64-bit
    pub endian: u8,   // 1=little, 2=big
    pub version: u8,
    pub os_abi: u8,
    pub abi_version: u8,
    pub e_type: u16,
    pub e_machine: u16,
    pub e_entry: u64,
}

/// ELF loader for binary execution.
pub struct ElfLoader {
    elf_header: Option<ELFHeader>,
}

impl ElfLoader {
    pub fn new() -> Self {
        Self {
            elf_header: None,
        }
    }

    /// Validate ELF magic bytes.
    pub fn validate_elf(data: &[u8]) -> bool {
        if data.len() < 4 {
            return false;
        }
        // ELF magic: 0x7f, 'E', 'L', 'F'
        data[0] == 0x7f && data[1] == b'E' && data[2] == b'L' && data[3] == b'F'
    }

    /// Parse ELF header from raw bytes.
    pub fn parse_header(&mut self, data: &[u8]) -> Result<()> {
        if !Self::validate_elf(data) {
            return Err(lucastra_core::LuCastraError::SyscallError(
                "Invalid ELF magic".to_string(),
            ));
        }

        if data.len() < 64 {
            return Err(lucastra_core::LuCastraError::SyscallError(
                "ELF header too small".to_string(),
            ));
        }

        let header = ELFHeader {
            magic: [data[0], data[1], data[2], data[3]],
            class: data[4],
            endian: data[5],
            version: data[6],
            os_abi: data[7],
            abi_version: data[8],
            e_type: u16::from_le_bytes([data[16], data[17]]),
            e_machine: u16::from_le_bytes([data[18], data[19]]),
            e_entry: u64::from_le_bytes([
                data[32], data[33], data[34], data[35], data[36], data[37], data[38], data[39],
            ]),
        };

        tracing::info!(
            "Parsed ELF header: class={}, machine={}, entry=0x{:x}",
            header.class,
            header.e_machine,
            header.e_entry
        );

        self.elf_header = Some(header);
        Ok(())
    }

    /// Get the entry point address.
    pub fn entry_point(&self) -> Option<u64> {
        self.elf_header.as_ref().map(|h| h.e_entry)
    }

    /// Load an ELF binary (stub for MVP).
    pub fn load(&mut self, data: &[u8]) -> Result<usize> {
        self.parse_header(data)?;
        tracing::info!("Loading ELF binary ({} bytes)", data.len());
        // Stub: return entry point as 0x1000
        Ok(0x1000)
    }
}

impl Default for ElfLoader {
    fn default() -> Self {
        Self::new()
    }
}
