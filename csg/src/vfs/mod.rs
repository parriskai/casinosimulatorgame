use std::io::{Seek, Write};
use glfw::Key::V;
use vfs::{FileSystem, VfsFileType};
use zip::write::{FileOptions, SimpleFileOptions};

use crate::errors::{GResult, GeneralizeError};

pub mod modularfs;
pub mod readonlyinclude;


pub fn join_path(a: &str, b: &str) -> String{
    if b.is_empty(){
        a.into()
    } else if a.ends_with('/'){
        String::from(a) + b
    } else {
        String::from(a) + "/" + b
    }
}

pub trait ExportVFS: FileSystem {
    fn save_to_zip<F: Write + Seek>(&self, file: F) -> GResult<()>{
        let mut zf = zip::write::ZipWriter::new(file);

        let mut queue = vec![String::new()];

        while !queue.is_empty(){
            //SAFTEY: We just checked
            let path = unsafe{queue.pop().unwrap_unchecked()};
            let metadata = self.metadata(&path).g_err()?;
            let mut buffer = [0; 1024 * 1024];

            match metadata.file_type {
                VfsFileType::File => {
                    zf.start_file_from_path(&path, SimpleFileOptions::default()).g_err()?;

                    let mut source = self.open_file(&path).g_err()?;
                    loop{
                        let size = source.read(&mut buffer).g_err()?;

                        // We could probably end if less than 2 ^ 20 bytes but the spec
                        // doest gurenteee that is the actual EOF
                        if size == 0{
                            break;
                        }

                        let wsize = zf.write(&buffer[..size]).g_err()?;

                        if size != wsize{
                            tracing::error!("VFS Export failed! Read {size} bytes but only wrote {wsize} bytes");
                        }
                    }
                }

                VfsFileType::Directory => {
                    zf.add_directory_from_path(&path, SimpleFileOptions::default());
                    queue.extend(self.read_dir(&path).g_err()?.map(|x| join_path(&path, x.as_str())));
                }
            }
        }

        Ok(())
    }
}

impl<T: FileSystem> ExportVFS for T{}