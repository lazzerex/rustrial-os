// Virtual File System abstraction layer

use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    File,
    Directory,
}

#[derive(Debug)]
pub enum VfsError {
    NotFound,
    AlreadyExists,
    NotADirectory,
    NotAFile,
    InvalidPath,
    NotInitialized,
    IoError,
}

impl core::fmt::Display for VfsError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            VfsError::NotFound => write!(f, "File or directory not found"),
            VfsError::AlreadyExists => write!(f, "File or directory already exists"),
            VfsError::NotADirectory => write!(f, "Not a directory"),
            VfsError::NotAFile => write!(f, "Not a file"),
            VfsError::InvalidPath => write!(f, "Invalid path"),
            VfsError::NotInitialized => write!(f, "Filesystem not initialized"),
            VfsError::IoError => write!(f, "I/O error"),
        }
    }
}

pub struct File {
    pub name: String,
    pub content: Vec<u8>,
    pub file_type: FileType,
}

impl File {
    pub fn new(name: String, content: Vec<u8>) -> Self {
        File {
            name,
            content,
            file_type: FileType::File,
        }
    }

    pub fn new_dir(name: String) -> Self {
        File {
            name,
            content: Vec::new(),
            file_type: FileType::Directory,
        }
    }

    pub fn is_file(&self) -> bool {
        self.file_type == FileType::File
    }

    pub fn is_dir(&self) -> bool {
        self.file_type == FileType::Directory
    }

    pub fn size(&self) -> usize {
        self.content.len()
    }

    pub fn read_to_string(&self) -> Result<String, VfsError> {
        String::from_utf8(self.content.clone())
            .map_err(|_| VfsError::IoError)
    }
}

pub struct Directory {
    pub name: String,
    pub entries: Vec<File>,
}

impl Directory {
    pub fn new(name: String) -> Self {
        Directory {
            name,
            entries: Vec::new(),
        }
    }

    pub fn add_file(&mut self, file: File) -> Result<(), VfsError> {
        if self.entries.iter().any(|f| f.name == file.name) {
            return Err(VfsError::AlreadyExists);
        }
        self.entries.push(file);
        Ok(())
    }

    pub fn find(&self, name: &str) -> Option<&File> {
        self.entries.iter().find(|f| f.name == name)
    }

    pub fn find_mut(&mut self, name: &str) -> Option<&mut File> {
        self.entries.iter_mut().find(|f| f.name == name)
    }

    pub fn remove(&mut self, name: &str) -> Result<(), VfsError> {
        if let Some(pos) = self.entries.iter().position(|f| f.name == name) {
            self.entries.remove(pos);
            Ok(())
        } else {
            Err(VfsError::NotFound)
        }
    }

    pub fn list(&self) -> &[File] {
        &self.entries
    }
}

pub trait FileSystem {
    fn create_file(&mut self, path: &str, content: &[u8]) -> Result<(), VfsError>;
    fn create_dir(&mut self, path: &str) -> Result<(), VfsError>;
    fn read_file(&self, path: &str) -> Result<Vec<u8>, VfsError>;
    fn read_file_to_string(&self, path: &str) -> Result<String, VfsError>;
    fn write_file(&mut self, path: &str, content: &[u8]) -> Result<(), VfsError>;
    fn delete(&mut self, path: &str) -> Result<(), VfsError>;
    fn exists(&self, path: &str) -> bool;
    fn is_file(&self, path: &str) -> bool;
    fn is_dir(&self, path: &str) -> bool;
    fn list_dir(&self, path: &str) -> Result<Vec<String>, VfsError>;
}
