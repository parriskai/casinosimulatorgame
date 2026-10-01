use vfs::{FileSystem, VfsFileType, VfsResult};
use zip::write::SimpleFileOptions;
use std::{fs::File, io::{Seek, Write}};
use crate::prelude::*;
use super::join_path;

pub trait CasinoFS: FileSystem {
    fn read_all<'a>(&self, path: &str) -> VfsResult<&'a [u8]>;

    fn save_to_zip(&self, path: &str) -> GResult<()>{
        let file = File::create(path).g_err()?;
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
                    zf.add_directory_from_path(&path, SimpleFileOptions::default()).g_err()?;
                    queue.extend(self.read_dir(&path).g_err()?.map(|x| join_path(&path, x.as_str())));
                }
            }
        }
        zf.finish().unwrap();
        Ok(())
    }
}