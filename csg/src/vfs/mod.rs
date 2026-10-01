pub mod readonlyinclude;
pub mod modularfs;
pub mod packed;
pub mod traits;

pub fn join_path(a: &str, b: &str) -> String{
    if b.is_empty(){
        a.into()
    } else if a.ends_with('/'){
        String::from(a) + b
    } else {
        String::from(a) + "/" + b
    }
}