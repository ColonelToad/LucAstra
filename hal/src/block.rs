use lucastra_core::Result;

/// Block device abstraction (USB drives, disks).
pub trait BlockDevice {
    fn read_sector(&mut self, sector: u64, buffer: &mut [u8]) -> Result<usize>;
    fn write_sector(&mut self, sector: u64, buffer: &[u8]) -> Result<usize>;
    fn sector_size(&self) -> usize;
    fn total_sectors(&self) -> u64;
}

/// In-memory mock block device for testing.
pub struct MockBlockDevice {
    data: Vec<u8>,
    sector_size: usize,
}

impl MockBlockDevice {
    pub fn new(size: usize, sector_size: usize) -> Self {
        Self {
            data: vec![0; size],
            sector_size,
        }
    }
}

impl BlockDevice for MockBlockDevice {
    fn read_sector(&mut self, sector: u64, buffer: &mut [u8]) -> Result<usize> {
        let offset = (sector as usize) * self.sector_size;
        let to_read = buffer.len().min(self.data.len() - offset);
        if offset < self.data.len() {
            buffer[..to_read].copy_from_slice(&self.data[offset..offset + to_read]);
            Ok(to_read)
        } else {
            Ok(0)
        }
    }

    fn write_sector(&mut self, sector: u64, buffer: &[u8]) -> Result<usize> {
        let offset = (sector as usize) * self.sector_size;
        let to_write = buffer.len().min(self.data.len() - offset);
        if offset < self.data.len() {
            self.data[offset..offset + to_write].copy_from_slice(&buffer[..to_write]);
            Ok(to_write)
        } else {
            Ok(0)
        }
    }

    fn sector_size(&self) -> usize {
        self.sector_size
    }

    fn total_sectors(&self) -> u64 {
        (self.data.len() / self.sector_size) as u64
    }
}
