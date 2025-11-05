// Filesystem module - Virtual File System abstraction

pub mod ramfs;
pub mod vfs;

pub use vfs::{FileSystem, File, Directory, FileType, VfsError};
pub use ramfs::RamFs;

use alloc::sync::Arc;
use spin::Mutex;

static ROOT_FS: Mutex<Option<Arc<Mutex<RamFs>>>> = Mutex::new(None);

/// Initialize the root filesystem
pub fn init() {
    let ramfs = RamFs::new();
    *ROOT_FS.lock() = Some(Arc::new(Mutex::new(ramfs)));
    crate::println!("[FS] Filesystem initialized");
}

/// Get a reference to the root filesystem
pub fn root_fs() -> Option<Arc<Mutex<RamFs>>> {
    ROOT_FS.lock().as_ref().cloned()
}

/// Mount a directory with files loaded from bootloader
pub fn mount_scripts(files: &[(&str, &[u8])]) -> Result<(), VfsError> {
    if let Some(fs) = root_fs() {
        let mut fs = fs.lock();
        
        // Create /scripts directory
        fs.create_dir("/scripts")?;
        
        // Add each script file
        for (name, content) in files {
            let path = alloc::format!("/scripts/{}", name);
            fs.create_file(&path, content)?;
            crate::println!("[FS] Mounted script: {}", path);
        }
        
        Ok(())
    } else {
        Err(VfsError::NotInitialized)
    }
}
