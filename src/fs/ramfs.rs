// RAM-based filesystem implementation

use super::vfs::{FileSystem, File, Directory, VfsError};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::collections::BTreeMap;

pub struct RamFs {
    root: Directory,
    // Cache for quick path lookups (reserved for future optimization)
    #[allow(dead_code)]
    path_cache: BTreeMap<String, Vec<String>>,
}

impl RamFs {
    pub fn new() -> Self {
        RamFs {
            root: Directory::new("/".to_string()),
            path_cache: BTreeMap::new(),
        }
    }

    /// Parse a path into components (returns owned Strings to avoid lifetime issues)
    fn parse_path(&self, path: &str) -> Result<Vec<String>, VfsError> {
        if !path.starts_with('/') {
            return Err(VfsError::InvalidPath);
        }

        let components: Vec<String> = path
            .trim_start_matches('/')
            .split('/')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        Ok(components)
    }

    /// Navigate to a directory given a path
    #[allow(dead_code)]
    fn navigate_to_dir(&self, path: &str) -> Result<&Directory, VfsError> {
        if path == "/" {
            return Ok(&self.root);
        }

        let components = self.parse_path(path)?;
        let _current_dir = &self.root;

        for component in &components {
            let entry = _current_dir
                .find(component)
                .ok_or(VfsError::NotFound)?;

            if !entry.is_dir() {
                return Err(VfsError::NotADirectory);
            }

            // For directory entries, we need to get subdirectories
            // This is a simplified approach; in reality, we'd need a more complex structure
            return Err(VfsError::NotADirectory);
        }

        Ok(_current_dir)
    }

    /// Navigate to a directory (mutable)
    #[allow(dead_code)]
    fn navigate_to_dir_mut(&mut self, path: &str) -> Result<&mut Directory, VfsError> {
        if path == "/" {
            return Ok(&mut self.root);
        }

        let _components = self.parse_path(path)?;
        
        // We need to track the directory hierarchy
        // For now, we'll use a simpler flat structure with full paths
        Err(VfsError::NotFound)
    }

    /// Get the parent path and file name
    #[allow(dead_code)]
    fn split_path(&self, path: &str) -> Result<(String, String), VfsError> {
        let components = self.parse_path(path)?;
        
        if components.is_empty() {
            return Err(VfsError::InvalidPath);
        }

        let filename = components.last().unwrap().clone();
        
        let parent_path = if components.len() == 1 {
            "/".to_string()
        } else {
            let parent_components = &components[..components.len() - 1];
            alloc::format!("/{}", parent_components.join("/"))
        };

        Ok((parent_path, filename))
    }

    /// Find a file entry by full path
    fn find_entry(&self, path: &str) -> Option<&File> {
        if path == "/" {
            return None;
        }

        let components = self.parse_path(path).ok()?;
        
        if components.is_empty() {
            return None;
        }

        // For a simple implementation, store files flat in root
        // with their full path as the name
        self.root.find(path)
    }

    /// Find a file entry by full path (mutable)
    fn find_entry_mut(&mut self, path: &str) -> Option<&mut File> {
        if path == "/" {
            return None;
        }

        let components = self.parse_path(path).ok()?;
        
        if components.is_empty() {
            return None;
        }

        self.root.find_mut(path)
    }
}

impl FileSystem for RamFs {
    fn create_file(&mut self, path: &str, content: &[u8]) -> Result<(), VfsError> {
        if self.exists(path) {
            return Err(VfsError::AlreadyExists);
        }

        let file = File::new(
            path.to_string(),
            content.to_vec(),
        );

        self.root.add_file(file)?;
        Ok(())
    }

    fn create_dir(&mut self, path: &str) -> Result<(), VfsError> {
        if self.exists(path) {
            return Err(VfsError::AlreadyExists);
        }

        let dir = File::new_dir(path.to_string());
        self.root.add_file(dir)?;
        Ok(())
    }

    fn read_file(&self, path: &str) -> Result<Vec<u8>, VfsError> {
        let file = self.find_entry(path).ok_or(VfsError::NotFound)?;
        
        if !file.is_file() {
            return Err(VfsError::NotAFile);
        }

        Ok(file.content.clone())
    }

    fn read_file_to_string(&self, path: &str) -> Result<String, VfsError> {
        let file = self.find_entry(path).ok_or(VfsError::NotFound)?;
        
        if !file.is_file() {
            return Err(VfsError::NotAFile);
        }

        file.read_to_string()
    }

    fn write_file(&mut self, path: &str, content: &[u8]) -> Result<(), VfsError> {
        if let Some(file) = self.find_entry_mut(path) {
            if !file.is_file() {
                return Err(VfsError::NotAFile);
            }
            file.content = content.to_vec();
            Ok(())
        } else {
            self.create_file(path, content)
        }
    }

    fn delete(&mut self, path: &str) -> Result<(), VfsError> {
        self.root.remove(path)
    }

    fn exists(&self, path: &str) -> bool {
        self.find_entry(path).is_some()
    }

    fn is_file(&self, path: &str) -> bool {
        self.find_entry(path)
            .map(|f| f.is_file())
            .unwrap_or(false)
    }

    fn is_dir(&self, path: &str) -> bool {
        if path == "/" {
            return true;
        }
        self.find_entry(path)
            .map(|f| f.is_dir())
            .unwrap_or(false)
    }

    fn list_dir(&self, path: &str) -> Result<Vec<String>, VfsError> {
        if path == "/" {
            // Return all entries in root
            return Ok(self.root.entries.iter()
                .map(|f| f.name.clone())
                .collect());
        }

        // For directories, find entries that start with this path
        let prefix = if path.ends_with('/') {
            path.to_string()
        } else {
            alloc::format!("{}/", path)
        };

        let mut entries = Vec::new();
        for file in &self.root.entries {
            if file.name.starts_with(&prefix) {
                // Get the relative name (just the next component)
                let relative = file.name.trim_start_matches(&prefix);
                if !relative.contains('/') {
                    entries.push(file.name.clone());
                }
            }
        }

        if entries.is_empty() && !self.is_dir(path) {
            return Err(VfsError::NotFound);
        }

        Ok(entries)
    }
}

impl Default for RamFs {
    fn default() -> Self {
        Self::new()
    }
}
