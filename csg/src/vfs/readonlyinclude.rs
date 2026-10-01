use std::{fmt::Debug, io::Cursor, iter::once};

use vfs::{FileSystem, VfsFileType, VfsMetadata, error::VfsErrorKind};

#[derive(Debug)]
pub enum FileType{
    Folder(&'static [&'static str]),
    File(&'static [u8])
} impl FileType{
    fn len(&self) -> Option<u64>{
        match self{
            FileType::Folder(_) => None,
            FileType::File(f) => Some(f.len() as u64)
        }
    }
} impl Into<VfsFileType> for &FileType{
    fn into(self) -> VfsFileType {
        match self {
            FileType::File(_) => VfsFileType::File,
            FileType::Folder(_) => VfsFileType::Directory
        }
    }
}

#[derive(Debug)]
pub struct ReadOnlyIncludeFS{
    pub data: &'static phf::Map<&'static str, FileType>
} impl FileSystem for ReadOnlyIncludeFS{
    fn read_dir(&self, path: &str) -> vfs::VfsResult<Box<dyn Iterator<Item = String> + Send>> {
        match self.data.get(path){
            None => Err(VfsErrorKind::FileNotFound.into()),

            Some(FileType::Folder(contents)) => {
                Ok(Box::new(contents.into_iter().map(|x| String::from(*x))))
            }

            Some(FileType::File(_)) => {
                Ok(Box::new(once(path.into())))
            }
        }
    }

    fn create_dir(&self, _: &str) -> vfs::VfsResult<()> {
        Err(VfsErrorKind::NotSupported.into())
    }

    fn open_file(&self, path: &str) -> vfs::VfsResult<Box<dyn vfs::SeekAndRead + Send>> {
        match self.data.get(path){
            None => Err(VfsErrorKind::FileNotFound.into()),

            Some(FileType::Folder(_)) => Err(VfsErrorKind::IoError(std::io::ErrorKind::IsADirectory.into()).into()),

            Some(FileType::File(contents)) => {
                Ok(Box::new(Cursor::new(*contents)))
            }
        }
    }

    fn create_file(&self, _path: &str) -> vfs::VfsResult<Box<dyn vfs::SeekAndWrite + Send>> {
        Err(VfsErrorKind::NotSupported.into())
    }

    fn append_file(&self, _path: &str) -> vfs::VfsResult<Box<dyn vfs::SeekAndWrite + Send>> {
        Err(VfsErrorKind::NotSupported.into())
    }

    fn metadata(&self, path: &str) -> vfs::VfsResult<vfs::VfsMetadata> {
        if let Some(file) = self.data.get(path){
            Ok(
                VfsMetadata{
                    file_type: file.into(),
                    len: file.len().unwrap_or(0),
                    created: None,
                    accessed: None,
                    modified: None
                }
            )
        } else {
            Err(VfsErrorKind::FileNotFound.into())
        }
    }

    fn exists(&self, path: &str) -> vfs::VfsResult<bool> {
        Ok(self.data.contains_key(path))
    }

    fn remove_file(&self, _path: &str) -> vfs::VfsResult<()> {
        Err(VfsErrorKind::NotSupported.into())
    }

    fn remove_dir(&self, _path: &str) -> vfs::VfsResult<()> {
        Err(VfsErrorKind::NotSupported.into())
    }

    fn set_creation_time(&self, _path: &str, _time: std::time::SystemTime) -> vfs::VfsResult<()> {
        Err(VfsErrorKind::NotSupported.into())
    }

    fn set_modification_time(&self, _path: &str, _time: std::time::SystemTime) -> vfs::VfsResult<()> {
        Err(VfsErrorKind::NotSupported.into())
    }

    fn set_access_time(&self, _path: &str, _time: std::time::SystemTime) -> vfs::VfsResult<()> {
        Err(VfsErrorKind::NotSupported.into())
    }

    fn copy_file(&self, _src: &str, _dest: &str) -> vfs::VfsResult<()> {
        Err(VfsErrorKind::NotSupported.into())
    }

    fn move_file(&self, _src: &str, _dest: &str) -> vfs::VfsResult<()> {
        Err(VfsErrorKind::NotSupported.into())
    }

    fn move_dir(&self, _src: &str, _dest: &str) -> vfs::VfsResult<()> {
        Err(VfsErrorKind::NotSupported.into())
    }
}