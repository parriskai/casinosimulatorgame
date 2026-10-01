include!(concat!(env!("OUT_DIR"), "/packed.rs"));

pub fn create_packed_vfs() -> ReadOnlyIncludeFS{
    ReadOnlyIncludeFS { data: &PACKED }
}