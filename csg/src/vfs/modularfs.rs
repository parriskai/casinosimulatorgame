use vfs::{FileSystem, VfsFileType, VfsMetadata, VfsResult, error::VfsErrorKind};
use std::collections::HashMap;

use crate::vfs::{join_path, traits::CasinoFS};

enum LocationType<'a>{
    Root,
    Mount(&'a Box<dyn CasinoFS>, &'a str),
    None
}
pub struct ModuleFS{
    mounts: HashMap<String, Box<dyn CasinoFS>>
} impl ModuleFS{
    pub fn create() -> ModuleFS{
        ModuleFS {
            mounts: HashMap::new()
        }
    }
    pub fn mount(&mut self, loc: String, fs: Box<dyn CasinoFS>) -> VfsResult<Option<Box<dyn CasinoFS>>>{
        if loc.contains('/') || loc == "." || loc == ".." {
            Err(VfsErrorKind::InvalidPath.into())
        } else {
            Ok(self.mounts.insert(loc, fs))
        }
    }

    pub fn umount(&mut self, loc: String) -> VfsResult<Box<dyn CasinoFS>>{
        if loc.contains('/') || loc == "." || loc == ".." {
            Err(VfsErrorKind::InvalidPath.into())
        } else {
            if let Some(mp) = self.mounts.remove(&loc){
                Ok(mp)
            } else {
                Err(VfsErrorKind::FileExists.into())
            }
        }
    }

    pub fn copy_file_across_mounts(smp: &Box<dyn CasinoFS>, srem: &str, dmp: &Box<dyn CasinoFS>, drem: &str) -> VfsResult<()>{
        let mut source = smp.open_file(srem)?;
        if dmp.exists(drem)?{
            dmp.remove_file(drem)?;
        }
        let mut destination = dmp.create_file(drem)?;

        let mut buffer = [0; 1024 * 1024];
        loop{
            let size = source.read(&mut buffer)?;

            // We could probably end if less than 2 ^ 20 bytes but the spec
            // doest gurenteee that is the actual EOF
            if size == 0{
                break;
            }

            let wsize = destination.write(&buffer[..size])?;

            if size != wsize{
                tracing::error!("VFS File copy failed! Read {size} bytes but only wrote {wsize} bytes");
            }
        }
        Ok(())
    }

    fn get<'a>(&'a self, path: &'a str) -> LocationType<'a>{
        let path = path.trim_start_matches('/');
        if path.is_empty(){
           return LocationType::Root;
        }
        // split mount/rest at the first `/` and remainder keeps the `/`
        let (mount, rem) = match path.find('/'){
            Some(i) => (&path[..i], &path[i..]),
            None => (path, ""),
        };
         
        match self.mounts.get(mount){
            Some(mp)=>LocationType::Mount(mp, rem), 
            None => LocationType::None,
        }
    }
} impl core::fmt::Debug for ModuleFS{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ModuleFS:\n")?;

        if self.mounts.is_empty(){
            f.write_str("| No mount points\n")?;
        }
        for (mp, fs) in self.mounts.iter(){
            f.write_fmt(format_args!("| /{mp} ({fs:?})\n"))?;
        }

        Ok(())
    }
} impl FileSystem for ModuleFS{
    fn read_dir(&self, path: &str) -> vfs::VfsResult<Box<dyn Iterator<Item = String> + Send>> {
        match self.get(path){
            LocationType::Root => {
                let keys: Vec<String> = self.mounts.keys().cloned().collect();
                Ok(Box::new(keys.into_iter()))
            }
            
            LocationType::Mount(mp, rem) => {
                mp.read_dir(rem)
            }

            LocationType::None => {
                Err(VfsErrorKind::FileNotFound.into())
            }
        }
    }

    fn create_dir(&self, path: &str) -> vfs::VfsResult<()> {
        match self.get(path) {
            LocationType::Root => {
                Err(VfsErrorKind::NotSupported.into())
            }

            LocationType::Mount(mp, rem) => {
                mp.create_dir(rem)
            }

            LocationType::None => {
                if path.contains('/'){
                    Err(VfsErrorKind::FileNotFound.into())
                } else {
                    Err(VfsErrorKind::NotSupported.into())
                }
            }
        }
    }

    fn open_file(&self, path: &str) -> vfs::VfsResult<Box<dyn vfs::SeekAndRead + Send>> {
        match self.get(path) {
            LocationType::Root => {
                Err(VfsErrorKind::NotSupported.into())
            }

            LocationType::Mount(mp, rem) => {
                mp.open_file(rem)
            }

            LocationType::None => {
                if path.contains('/'){
                    Err(VfsErrorKind::FileNotFound.into())
                } else {
                    Err(VfsErrorKind::NotSupported.into())
                }
            }
        }
    }

    fn create_file(&self, path: &str) -> vfs::VfsResult<Box<dyn vfs::SeekAndWrite + Send>> {
        match self.get(path) {
            LocationType::Root => {
                Err(VfsErrorKind::NotSupported.into())
            }

            LocationType::Mount(mp, rem) => {
                mp.create_file(rem)
            }

            LocationType::None => {
                if path.contains('/'){
                    Err(VfsErrorKind::FileNotFound.into())
                } else {
                    Err(VfsErrorKind::NotSupported.into())
                }
            }
        }
    }

    fn append_file(&self, path: &str) -> vfs::VfsResult<Box<dyn vfs::SeekAndWrite + Send>> {
        match self.get(path) {
            LocationType::Root => {
                Err(VfsErrorKind::NotSupported.into())
            }

            LocationType::Mount(mp, rem) => {
                mp.append_file(rem)
            }

            LocationType::None => {
                if path.contains('/'){
                    Err(VfsErrorKind::FileNotFound.into())
                } else {
                    Err(VfsErrorKind::NotSupported.into())
                }
            }
        }
    }

    fn metadata(&self, path: &str) -> vfs::VfsResult<vfs::VfsMetadata> {
        match self.get(path) {
            LocationType::Root => {
                Ok(VfsMetadata{
                    file_type: vfs::VfsFileType::Directory,
                    len: 0,
                    created: None,
                    modified: None,
                    accessed: None
                })
            }

            LocationType::Mount(mp, rem) => {
                mp.metadata(rem)
            }

            LocationType::None => {
                Err(VfsErrorKind::FileNotFound.into())
            }
        }
    }

    fn exists(&self, path: &str) -> vfs::VfsResult<bool> {
        match self.get(path) {
            LocationType::Root => {
                Ok(true)
            }

            LocationType::Mount(mp, rem) => {
                mp.exists(rem)
            }

            LocationType::None => {
                if path.contains('/'){
                    Err(VfsErrorKind::FileNotFound.into())
                } else {
                    Ok(false)
                }
            }
        }
    }

    fn remove_file(&self, path: &str) -> vfs::VfsResult<()> {
        match self.get(path) {
            LocationType::Root => {
                Err(VfsErrorKind::NotSupported.into())
            }

            LocationType::Mount(mp, rem) => {
                mp.remove_file(rem)
            }

            LocationType::None => {
                if path.contains('/'){
                    Err(VfsErrorKind::FileNotFound.into())
                } else {
                    Err(VfsErrorKind::NotSupported.into())
                }
            }
        }
    }

    fn remove_dir(&self, path: &str) -> vfs::VfsResult<()> {
        match self.get(path) {
            LocationType::Root => {
                Err(VfsErrorKind::NotSupported.into())
            }

            LocationType::Mount(mp, rem) => {
                mp.remove_dir(rem)
            }

            LocationType::None => {
                if path.contains('/'){
                    Err(VfsErrorKind::FileNotFound.into())
                } else {
                    Err(VfsErrorKind::NotSupported.into())
                }
            }
        }
    }

    fn set_creation_time(&self, path: &str, time: std::time::SystemTime) -> vfs::VfsResult<()> {
        match self.get(path) {
            LocationType::Root => {
                Err(VfsErrorKind::NotSupported.into())
            }

            LocationType::Mount(mp, rem) => {
                mp.set_creation_time(rem, time)
            }

            LocationType::None => {
                if path.contains('/'){
                    Err(VfsErrorKind::FileNotFound.into())
                } else {
                    Err(VfsErrorKind::NotSupported.into())
                }
            }
        }
    }

    fn set_modification_time(&self, path: &str, time: std::time::SystemTime) -> vfs::VfsResult<()> {
        match self.get(path) {
            LocationType::Root => {
                Err(VfsErrorKind::NotSupported.into())
            }

            LocationType::Mount(mp, rem) => {
                mp.set_modification_time(rem, time)
            }

            LocationType::None => {
                if path.contains('/'){
                    Err(VfsErrorKind::FileNotFound.into())
                } else {
                    Err(VfsErrorKind::NotSupported.into())
                }
            }
        }
    }

    fn set_access_time(&self, path: &str, time: std::time::SystemTime) -> vfs::VfsResult<()> {
        match self.get(path) {
            LocationType::Root => {
                Err(VfsErrorKind::NotSupported.into())
            }

            LocationType::Mount(mp, rem) => {
                mp.set_access_time(rem, time)
            }

            LocationType::None => {
                if path.contains('/'){
                    Err(VfsErrorKind::FileNotFound.into())
                } else {
                    Err(VfsErrorKind::NotSupported.into())
                }
            }
        }
    }

    fn copy_file(&self, src: &str, dest: &str) -> vfs::VfsResult<()> {
        match (self.get(src), self.get(dest)) {
            (LocationType::Root, _) | (_, LocationType::Root) => {
                Err(VfsErrorKind::NotSupported.into())
            },

            (LocationType::Mount(smp, srem), LocationType::Mount(dmp, drem)) => {
                if std::ptr::eq(smp.as_ref(), dmp.as_ref()){
                    smp.copy_file(&srem, &drem)
                } else {
                    Self::copy_file_across_mounts(smp, &srem, dmp, &drem)
                }
            }
            (LocationType::None, _) => {
                if src.contains('/'){
                    Err(VfsErrorKind::FileNotFound.into())
                } else {
                    Err(VfsErrorKind::NotSupported.into())
                }
            },

            (_, LocationType::None) => {
                if dest.contains('/'){
                    Err(VfsErrorKind::FileNotFound.into())
                } else {
                    Err(VfsErrorKind::NotSupported.into())
                }
            }
        }
    }

    fn move_file(&self, src: &str, dest: &str) -> vfs::VfsResult<()> {
        match (self.get(src), self.get(dest)) {
            (LocationType::Root, _) | (_, LocationType::Root) => {
                Err(VfsErrorKind::NotSupported.into())
            },

            (LocationType::Mount(smp, srem), LocationType::Mount(dmp, drem)) => {
                if std::ptr::eq(smp.as_ref(), dmp.as_ref()){
                    smp.move_file(&srem, &drem)
                } else {
                    Self::copy_file_across_mounts(smp, &srem, dmp, &drem)?;
                    smp.remove_file(&srem)?;
                    Ok(())
                }
            }
            (LocationType::None, _) => {
                if src.contains('/'){
                    Err(VfsErrorKind::FileNotFound.into())
                } else {
                    Err(VfsErrorKind::NotSupported.into())
                }
            },

            (_, LocationType::None) => {
                if dest.contains('/'){
                    Err(VfsErrorKind::FileNotFound.into())
                } else {
                    Err(VfsErrorKind::NotSupported.into())
                }
            }
        }
    }

    fn move_dir(&self, src: &str, dest: &str) -> VfsResult<()> {
        match (self.get(src), self.get(dest)) {
            (LocationType::Root, _) | (_, LocationType::Root) => {
                Err(VfsErrorKind::NotSupported.into())
            },

            (LocationType::Mount(smp, srem), LocationType::Mount(dmp, drem)) => {
                if std::ptr::eq(smp.as_ref(), dmp.as_ref()){
                    smp.move_dir(&srem, &drem)
                } else {
                    let mut queue = vec![String::new()];

                    while !queue.is_empty(){
                        //SAFTEY: We just checked
                        let path = unsafe{queue.pop().unwrap_unchecked()};
                        let origin = &join_path(&srem, &path);
                        let destination = &join_path(&drem, &path);

                        let metadata = smp.metadata(origin)?;
                        
                        match metadata.file_type {
                            VfsFileType::File => {
                                Self::copy_file_across_mounts(smp, origin, dmp, destination)?;
                            }
                            VfsFileType::Directory => {
                                dmp.create_dir(destination)?;

                                queue.extend(smp.read_dir(origin)?.map(|x| join_path(&path, x.as_str())));
                            }
                        }
                    }
                    Ok(())
                }
            }
            (LocationType::None, _) => {
                if src.contains('/'){
                    Err(VfsErrorKind::FileNotFound.into())
                } else {
                    Err(VfsErrorKind::NotSupported.into())
                }
            },

            (_, LocationType::None) => {
                if dest.contains('/'){
                    Err(VfsErrorKind::FileNotFound.into())
                } else {
                    Err(VfsErrorKind::NotSupported.into())
                }
            }
        }
    }
} impl CasinoFS for ModuleFS{
    fn should_reload(&self) -> bool {
        self.mounts.iter().any(|(_, mt)| mt.should_reload())
    }
    
    fn read_all<'a>(&self, path: &str) -> VfsResult<&'a [u8]> {
        match self.get(path) {
            LocationType::Root => {
                Err(VfsErrorKind::NotSupported.into())
            }

            LocationType::Mount(mp, rem) => {
                mp.read_all(rem)
            }

            LocationType::None => {
                if path.contains('/'){
                    Err(VfsErrorKind::FileNotFound.into())
                } else {
                    Err(VfsErrorKind::NotSupported.into())
                }
            }
        }
    }
}