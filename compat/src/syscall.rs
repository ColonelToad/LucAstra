//! Syscall stubs for POSIX compatibility.
//!
//! Maps common Linux syscalls to LucAstra kernel operations.
//! Focuses on file I/O, device I/O, and basic process control.

use lucastra_core::Result;
use std::collections::HashMap;

/// File descriptor table for a process.
pub struct FileDescriptorTable {
    fds: HashMap<i32, FileDescriptor>,
    next_fd: i32,
}

#[derive(Clone, Debug)]
pub struct FileDescriptor {
    pub path: String,
    pub flags: i32,
    pub offset: u64,
}

impl FileDescriptorTable {
    pub fn new() -> Self {
        let mut fds = HashMap::new();
        // Standard file descriptors
        fds.insert(
            0,
            FileDescriptor {
                path: "stdin".to_string(),
                flags: 0,
                offset: 0,
            },
        );
        fds.insert(
            1,
            FileDescriptor {
                path: "stdout".to_string(),
                flags: 1,
                offset: 0,
            },
        );
        fds.insert(
            2,
            FileDescriptor {
                path: "stderr".to_string(),
                flags: 1,
                offset: 0,
            },
        );

        Self { fds, next_fd: 3 }
    }

    pub fn open(&mut self, path: &str, flags: i32) -> i32 {
        let fd = self.next_fd;
        self.fds.insert(
            fd,
            FileDescriptor {
                path: path.to_string(),
                flags,
                offset: 0,
            },
        );
        self.next_fd += 1;
        fd
    }

    pub fn close(&mut self, fd: i32) -> bool {
        self.fds.remove(&fd).is_some()
    }

    pub fn get(&self, fd: i32) -> Option<&FileDescriptor> {
        self.fds.get(&fd)
    }

    pub fn get_mut(&mut self, fd: i32) -> Option<&mut FileDescriptor> {
        self.fds.get_mut(&fd)
    }

    pub fn dup(&mut self, fd: i32) -> Option<i32> {
        if let Some(desc) = self.fds.get(&fd) {
            let new_fd = self.next_fd;
            self.fds.insert(new_fd, desc.clone());
            self.next_fd += 1;
            Some(new_fd)
        } else {
            None
        }
    }
}

impl Default for FileDescriptorTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Syscall handler for POSIX compatibility.
pub struct SyscallHandler {
    fd_table: FileDescriptorTable,
    // In-memory file storage for testing (path -> content)
    pub file_data: HashMap<String, Vec<u8>>,
}

impl SyscallHandler {
    pub fn new() -> Self {
        Self {
            fd_table: FileDescriptorTable::new(),
            file_data: HashMap::new(),
        }
    }

    /// Store file data for read testing.
    pub fn write_file(&mut self, path: &str, data: Vec<u8>) {
        self.file_data.insert(path.to_string(), data);
    }

    /// Open a file by registered path (for testing/direct calls).
    pub fn open_file(&mut self, path: &str, flags: i32) -> i32 {
        self.fd_table.open(path, flags)
    }

    /// Handle a syscall by number and arguments.
    /// Returns the syscall result or an error.
    pub fn handle_syscall(&mut self, syscall_num: u64, args: &[u64]) -> Result<i64> {
        match syscall_num {
            // open(const char *path, int flags, mode_t mode)
            2 => {
                let path_ptr = args.get(0).copied().unwrap_or(0) as usize;
                let flags = args.get(1).copied().unwrap_or(0) as i32;
                // Mock: just return a new fd
                let path = format!("/path/{}", path_ptr);
                let fd = self.fd_table.open(&path, flags);
                tracing::debug!("syscall: open({}) -> fd {}", path, fd);
                Ok(fd as i64)
            }
            // read(int fd, void *buf, size_t count)
            0 => {
                let fd = args.get(0).copied().unwrap_or(0) as i32;
                let count = args.get(2).copied().unwrap_or(0) as usize;

                if let Some(desc) = self.fd_table.get_mut(fd) {
                    if let Some(data) = self.file_data.get(&desc.path) {
                        let start = desc.offset as usize;
                        let end = (start + count).min(data.len());
                        let bytes_read = (end - start).max(0);
                        desc.offset += bytes_read as u64;
                        tracing::debug!(
                            "syscall: read(fd={}, count={}) -> {} bytes",
                            fd,
                            count,
                            bytes_read
                        );
                        return Ok(bytes_read as i64);
                    }
                }
                tracing::debug!("syscall: read(fd={}) -> 0 (not found or EOF)", fd);
                Ok(0)
            }
            // write(int fd, const void *buf, size_t count)
            1 => {
                let fd = args.get(0).copied().unwrap_or(0) as i32;
                let count = args.get(2).copied().unwrap_or(0) as usize;

                if let Some(desc) = self.fd_table.get(fd) {
                    tracing::debug!(
                        "syscall: write(fd={}, path={}, count={})",
                        fd,
                        desc.path,
                        count
                    );
                    Ok(count as i64)
                } else {
                    tracing::debug!("syscall: write(fd={}) -> EBADF", fd);
                    Ok(-9) // EBADF
                }
            }
            // close(int fd)
            3 => {
                let fd = args.get(0).copied().unwrap_or(0) as i32;
                let result = if self.fd_table.close(fd) { 0 } else { -9 };
                tracing::debug!("syscall: close(fd={}) -> {}", fd, result);
                Ok(result)
            }
            // lseek(int fd, off_t offset, int whence)
            8 => {
                let fd = args.get(0).copied().unwrap_or(0) as i32;
                let offset = args.get(1).copied().unwrap_or(0) as i64;
                let whence = args.get(2).copied().unwrap_or(0) as i32;

                if let Some(desc) = self.fd_table.get_mut(fd) {
                    match whence {
                        0 => desc.offset = offset as u64, // SEEK_SET
                        1 => desc.offset = (desc.offset as i64 + offset) as u64, // SEEK_CUR
                        2 => {
                            // SEEK_END
                            if let Some(data) = self.file_data.get(&desc.path) {
                                desc.offset = (data.len() as i64 + offset) as u64;
                            }
                        }
                        _ => {}
                    }
                    tracing::debug!(
                        "syscall: lseek(fd={}, offset={}, whence={}) -> {}",
                        fd,
                        offset,
                        whence,
                        desc.offset
                    );
                    Ok(desc.offset as i64)
                } else {
                    Ok(-9)
                }
            }
            // dup(int oldfd)
            32 => {
                let fd = args.get(0).copied().unwrap_or(0) as i32;
                if let Some(new_fd) = self.fd_table.dup(fd) {
                    tracing::debug!("syscall: dup({}) -> {}", fd, new_fd);
                    Ok(new_fd as i64)
                } else {
                    Ok(-9)
                }
            }
            // ioctl(int fd, unsigned long request, ...)
            16 => {
                let fd = args.get(0).copied().unwrap_or(0) as i32;
                let request = args.get(1).copied().unwrap_or(0);
                tracing::debug!("syscall: ioctl(fd={}, request=0x{:x}) -> 0", fd, request);
                Ok(0)
            }
            // exit(int status)
            60 => {
                let status = args.get(0).copied().unwrap_or(0) as i32;
                tracing::info!("syscall: exit with status {}", status);
                Ok(0)
            }
            // mmap (stub)
            9 => {
                tracing::debug!("syscall: mmap (stub) -> 0x1000");
                Ok(0x1000)
            }
            // brk (stub)
            12 => {
                let addr = args.get(0).copied().unwrap_or(0);
                tracing::debug!("syscall: brk(0x{:x}) -> 0x{:x}", addr, addr);
                Ok(addr as i64)
            }
            _ => {
                tracing::debug!("syscall: unknown syscall {}", syscall_num);
                Ok(-38) // ENOSYS
            }
        }
    }
}

impl Default for SyscallHandler {
    fn default() -> Self {
        Self::new()
    }
}
