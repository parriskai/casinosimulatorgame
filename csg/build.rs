//! Build.rs, runs at compile time to generate the included $(OUT_DIR)/packed.rs file
//! [`csg::packed.rs`] includes this file and thus the files defined in packed.json can be used
//! from whithin the VFS structure

use std::{
    collections::HashMap,
    env,
    fs::{
        self,
        File
    },
    io::Write,
    path::Path
};
use indexmap::IndexMap;
use serde::Deserialize;

/// Listing for one folder
#[derive(Debug, Deserialize)]
struct IncludeListing{
    /// The location mounted to in the vfs
    loc: String,
    /// Can the contents be overwritten by other includes false: yes, true: no (default false)
    #[serde(default)]
    #[serde(rename="final")]
    fin: bool
}

/// Join two path strings, does not perform . and .. expansion, only merge with a / in between if the first does not include one
pub fn join_path(a: &str, b: &str) -> String{
    if b.is_empty(){ // If b is empty return a
        a.into()
    } else if a.ends_with('/'){
        String::from(a) + b
    } else {
        String::from(a) + "/" + b
    }
}

/// Atempt to parse the packed.json file stored in the crate root
/// Returns
///  - Some(IndexMap<_>) on sucsessful read
///  - None on error
fn read_packed() -> Option<IndexMap<String, IncludeListing>>{
    let file = match std::fs::read_to_string("packed.json"){
        Ok(contents) => Some(contents),
        Err(e) => {
            println!("cargo::error=Failed to read packed.json: {e}");
            None
        }
    }?;

    match serde_json::from_str(&file){
        Ok(r) => Some(r),
        Err(e) => {
            println!("cargo::error=Failed to parse packed.json: {e}");
            None
        }
    }
}

/// Filetype enum
#[derive(Debug)]
enum FileType{
    /// A directory containing a Vec of contents (file name only not path)
    Directory(Vec<String>),
    /// A single file stored on disk
    File
}

/// Try to remove a folder and its contents when overriden by a string
/// Returns:
///  - true on sucsessful deletion
///  - false on failure (one or more files are marked readonly)
fn try_remove_override(listing: &mut HashMap<String, (String, bool, FileType)>, folder: String) -> bool{
    let mut need_to_remove = Vec::new();

    // The file tree is first recursivly walked, listing out all files to remove, aborting if any are final
    let mut queue = vec![folder.clone()];
    while !queue.is_empty(){
        // SAFTEY: We already know queue is not empty
        let entry = unsafe{queue.pop().unwrap_unchecked()};
        
        // If it exists in the listing
        if let Some((physical, fin, file_type)) = listing.get(&entry){
            // Found a final file
            if *fin{
                println!("cargo::warning={entry} ({physical}) is final and cant be overwritten by {folder}");
                return false;
            }

            match file_type{
                FileType::Directory(d) => {
                    // Remove the folder
                    need_to_remove.push(entry.clone());
                    // And its contents
                    queue.extend(d.iter().map(|content| join_path(&entry, content)));
                }
                FileType::File => {
                    // Remove the file
                    need_to_remove.push(entry);
                }
            }
        }
    }

    // Once we have found all files and are sure none are final
    for entry in need_to_remove{
        listing.remove(&entry);
    }
    true
}

fn include_in_listing(listing: &mut HashMap<String, (String, bool, FileType)>, physical: &Path, vfs: String, fin: bool) -> bool{
    println!("cargo::rerun-if-changed={}", physical.to_str().unwrap());
    if physical.try_exists().unwrap_or_default(){
        if physical.is_dir(){
            let mut index = if let Some((_, _, old_t)) = listing.get(&vfs){
                match old_t{
                    FileType::Directory(d) => {
                        // It works, dont question it too much
                        d.clone()
                    }
                    FileType::File => {
                        listing.remove(&vfs);
                        Vec::new()
                    }
                }
            } else {
                Vec::new()
            };

            for f in physical.read_dir().unwrap().map(|x| x.unwrap()){
                if include_in_listing(listing, &f.path(), join_path(&vfs, &f.file_name().to_string_lossy()), fin){
                    index.push(f.file_name().to_string_lossy().into_owned());
                }
            }
            listing.insert(vfs, (physical.to_string_lossy().to_string(), fin, FileType::Directory(index)));
            return true;
        } else if physical.is_file(){
            if let Some((old_path, old_fin, old_t)) = listing.get(&vfs){
                if *old_fin{
                    println!("cargo::warning={vfs} ({old_path}) is marked final so the path {physical:?} cannot be written over it!");
                    return false;
                }

                match old_t{
                    FileType::Directory(_) => {
                        if !try_remove_override(listing, vfs.clone()){
                            return false;
                        }
                    }

                    FileType::File => {}
                }
            }
            
            listing.insert(vfs, (physical.to_string_lossy().to_string(), fin, FileType::File));
            return true;
        } else {
            println!("cargo::warning={} is not a directory or file, it will be excluded", physical.to_str().unwrap());
            return false;
        }
    } else {
        println!("cargo::warning={} does not exist or cannot be read", physical.to_str().unwrap());
        return false;
    }
}

fn main(){
    println!("cargo::rerun-if-changed=packed.json");
    let pack = read_packed().unwrap_or_default();
    let mut table = HashMap::new();
    for (physical, listing) in pack{
        let physical = fs::canonicalize(physical).unwrap();
        include_in_listing(&mut table, physical.as_path(), listing.loc, listing.fin);
    }

    let out_dir = env::var("OUT_DIR").unwrap();
    let generated_path = Path::new(&out_dir).join("packed.rs");

    let mut out_file = File::create(generated_path).unwrap();
    out_file.write(b"use crate::vfs::readonlyinclude::{ReadOnlyIncludeFS, FileType};\n").unwrap();
    out_file.write(b"use phf::phf_map;\n").unwrap();
    out_file.write(b"\n").unwrap();
    out_file.write(b"static PACKED: phf::Map<&'static str, FileType> = phf_map!{\n").unwrap();

    let mut items: Vec<_> = table.into_iter().collect();

    if items.is_empty(){
        out_file.write(b"    /* No files */").unwrap();
    } else{
        items.sort_by(|a, b| a.0.cmp(&b.0));
        let mut first = true;
        for (name, (phys, _, ft)) in items{
            if !first{
                out_file.write(b",\n").unwrap();
            }

            out_file.write_fmt(format_args!("    {name:?} => FileType::")).unwrap();
            match ft {
                FileType::Directory(mut contents) => {
                    out_file.write(b"Folder(&[").unwrap();
                    
                    if contents.is_empty(){
                        out_file.write(b"/* No files */").unwrap();
                    } else {
                        contents.sort();
                        let mut dir_first = true;
                        for idx in contents{
                            if !dir_first{
                                out_file.write(b", ").unwrap();
                            }
                            out_file.write_fmt(format_args!("{idx:?}")).unwrap();
                            dir_first = false;
                        }
                    }

                    out_file.write(b"])").unwrap();
                },

                FileType::File => {
                    out_file.write_fmt(format_args!("File(include_bytes!({phys:?}))")).unwrap();
                }
            }

            first = false;
        }
    }
    out_file.write(b"\n").unwrap();
    out_file.write(b"};").unwrap();
    // FIN
}