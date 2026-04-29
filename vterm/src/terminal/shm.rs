use rustc_hash::FxHasher;
use std::hash::Hasher;
use std::ptr;
use std::sync::atomic::{AtomicU32, Ordering};

// SHM Layout:
// 0..4: Magic (0x5654524D "VTRM")
// 4..8: Sequence Number (u32)
// 8..1032: Bloom Filter (8192 bits = 1024 bytes)
// 1032..4096: Screen Content (Rendered text)

pub const SHM_MAGIC: u32 = 0x5654524D;
pub const SHM_SEQ_OFFSET: usize = 4;
pub const SHM_BLOOM_OFFSET: usize = 8;
pub const SHM_BLOOM_SIZE: usize = 1024;
pub const SHM_SCREEN_OFFSET: usize = 1032;

#[cfg(windows)]
mod imp {
    use super::*;
    use windows_sys::Win32::Foundation::{
        CloseHandle, HANDLE, INVALID_HANDLE_VALUE, WAIT_OBJECT_0,
    };
    use windows_sys::Win32::System::Memory::*;
    use windows_sys::Win32::System::Threading::*;

    pub struct ShmBuffer {
        handle: HANDLE,
        event: HANDLE,
        ptr: *mut u8,
        size: usize,
        seq: AtomicU32,
    }

    impl ShmBuffer {
        pub fn new(name: &str, size: usize) -> crate::Result<Self> {
            Self::create_or_open(name, size, true)
        }

        pub fn open_existing(name: &str, size: usize) -> crate::Result<Self> {
            Self::create_or_open(name, size, false)
        }

        fn create_or_open(name: &str, size: usize, create: bool) -> crate::Result<Self> {
            let name_wide: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();
            unsafe {
                let handle = if create {
                    CreateFileMappingW(
                        INVALID_HANDLE_VALUE,
                        ptr::null_mut(),
                        PAGE_READWRITE,
                        0,
                        size as u32,
                        name_wide.as_ptr(),
                    )
                } else {
                    OpenFileMappingW(FILE_MAP_READ, 0, name_wide.as_ptr())
                };

                if handle == 0 {
                    return Err(crate::Error::Pty(format!(
                        "{} failed: {}",
                        if create {
                            "CreateFileMappingW"
                        } else {
                            "OpenFileMappingW"
                        },
                        std::io::Error::last_os_error()
                    )));
                }

                let addr = MapViewOfFile(
                    handle,
                    if create {
                        FILE_MAP_ALL_ACCESS
                    } else {
                        FILE_MAP_READ
                    },
                    0,
                    0,
                    size,
                );
                if addr.Value.is_null() {
                    CloseHandle(handle);
                    return Err(crate::Error::Pty(format!(
                        "MapViewOfFile failed: {}",
                        std::io::Error::last_os_error()
                    )));
                }

                let ptr = addr.Value as *mut u8;
                if create {
                    // Initialize header
                    ptr::write(ptr as *mut u32, SHM_MAGIC);
                    ptr::write(ptr.add(SHM_SEQ_OFFSET) as *mut u32, 0);
                    ptr::write_bytes(ptr.add(SHM_BLOOM_OFFSET), 0, SHM_BLOOM_SIZE);
                }

                let event_name = format!("{}-evt", name);
                let event_name_wide: Vec<u16> = event_name.encode_utf16().chain(Some(0)).collect();
                let event = if create {
                    CreateEventW(
                        ptr::null_mut(),
                        0, // manual reset: false
                        0, // initial state: false
                        event_name_wide.as_ptr(),
                    )
                } else {
                    OpenEventW(EVENT_ALL_ACCESS, 0, event_name_wide.as_ptr())
                };

                if event == 0 {
                    UnmapViewOfFile(MEMORY_MAPPED_VIEW_ADDRESS {
                        Value: ptr as *mut _,
                    });
                    CloseHandle(handle);
                    return Err(crate::Error::Pty(format!(
                        "{} failed: {}",
                        if create { "CreateEventW" } else { "OpenEventW" },
                        std::io::Error::last_os_error()
                    )));
                }

                Ok(Self {
                    handle,
                    event,
                    ptr,
                    size,
                    seq: AtomicU32::new(0),
                })
            }
        }

        pub fn update_header(&self) {
            let next = self.seq.fetch_add(1, Ordering::Relaxed) + 1;
            unsafe {
                ptr::write_volatile(self.ptr.add(SHM_SEQ_OFFSET) as *mut u32, next);
            }
        }

        pub fn clear_bloom(&self) {
            unsafe {
                ptr::write_bytes(self.ptr.add(SHM_BLOOM_OFFSET), 0, SHM_BLOOM_SIZE);
            }
        }

        pub fn insert_token(&self, token: &str) {
            if token.is_empty() {
                return;
            }

            let mut h1 = FxHasher::default();
            h1.write(token.as_bytes());
            let hash1 = h1.finish();

            let mut h2 = FxHasher::default();
            h2.write_u64(hash1);
            h2.write_u8(1);
            let hash2 = h2.finish();

            let mut h3 = FxHasher::default();
            h3.write_u64(hash1);
            h3.write_u8(2);
            let hash3 = h3.finish();

            for h in [hash1, hash2, hash3] {
                let bit_idx = (h % (SHM_BLOOM_SIZE as u64 * 8)) as usize;
                let byte_idx = bit_idx / 8;
                let bit_mask = 1 << (bit_idx % 8);
                unsafe {
                    let target = self.ptr.add(SHM_BLOOM_OFFSET + byte_idx);
                    let current = ptr::read_volatile(target);
                    ptr::write_volatile(target, current | bit_mask);
                }
            }
        }

        pub fn check_bloom(&self, token: &str) -> bool {
            if token.is_empty() {
                return false;
            }

            let mut h1 = FxHasher::default();
            h1.write(token.as_bytes());
            let hash1 = h1.finish();

            let mut h2 = FxHasher::default();
            h2.write_u64(hash1);
            h2.write_u8(1);
            let hash2 = h2.finish();

            let mut h3 = FxHasher::default();
            h3.write_u64(hash1);
            h3.write_u8(2);
            let hash3 = h3.finish();

            for h in [hash1, hash2, hash3] {
                let bit_idx = (h % (SHM_BLOOM_SIZE as u64 * 8)) as usize;
                let byte_idx = bit_idx / 8;
                let bit_mask = 1 << (bit_idx % 8);
                unsafe {
                    let target = self.ptr.add(SHM_BLOOM_OFFSET + byte_idx);
                    let current = ptr::read_volatile(target);
                    if (current & bit_mask) == 0 {
                        return false;
                    }
                }
            }
            true
        }

        pub fn write_screen(&self, data: &[u8]) {
            let max_len = self.size - SHM_SCREEN_OFFSET;
            let len = data.len().min(max_len);
            unsafe {
                ptr::copy_nonoverlapping(data.as_ptr(), self.ptr.add(SHM_SCREEN_OFFSET), len);
                if len < max_len {
                    ptr::write(self.ptr.add(SHM_SCREEN_OFFSET + len), 0);
                }
                SetEvent(self.event);
            }
        }

        pub fn wait_for_change(&self, timeout_ms: u32) -> bool {
            unsafe {
                let res = WaitForSingleObject(self.event, timeout_ms);
                res == WAIT_OBJECT_0
            }
        }

        pub fn read_screen(&self) -> String {
            unsafe {
                let ptr = self.ptr.add(SHM_SCREEN_OFFSET);
                let mut len = 0;
                let max_len = self.size - SHM_SCREEN_OFFSET;
                while len < max_len && *ptr.add(len) != 0 {
                    len += 1;
                }
                String::from_utf8_lossy(std::slice::from_raw_parts(ptr, len)).to_string()
            }
        }
    }

    impl Drop for ShmBuffer {
        fn drop(&mut self) {
            unsafe {
                UnmapViewOfFile(MEMORY_MAPPED_VIEW_ADDRESS {
                    Value: self.ptr as *mut _,
                });
                CloseHandle(self.event);
                CloseHandle(self.handle);
            }
        }
    }

    unsafe impl Send for ShmBuffer {}
    unsafe impl Sync for ShmBuffer {}
}

#[cfg(not(windows))]
mod imp {
    use super::*;

    pub struct ShmBuffer {
        seq: AtomicU32,
    }

    impl ShmBuffer {
        pub fn new(_name: &str, _size: usize) -> crate::Result<Self> {
            Err(crate::Error::Pty("SHM is only supported on Windows".into()))
        }

        pub fn open_existing(_name: &str, _size: usize) -> crate::Result<Self> {
            Err(crate::Error::Pty("SHM is only supported on Windows".into()))
        }

        pub fn update_header(&self) {
            self.seq.fetch_add(1, Ordering::Relaxed);
        }

        pub fn clear_bloom(&self) {}
        pub fn insert_token(&self, _token: &str) {}
        pub fn check_bloom(&self, _token: &str) -> bool {
            false
        }
        pub fn write_screen(&self, _data: &[u8]) {}
        pub fn wait_for_change(&self, _timeout_ms: u32) -> bool {
            false
        }
        pub fn read_screen(&self) -> String {
            String::new()
        }
    }
}

pub use imp::ShmBuffer;
