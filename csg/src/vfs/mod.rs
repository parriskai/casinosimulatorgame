use std::{collections::HashMap, fmt::{Display, Write}, iter::once};


enum Index<D,F>{
    Directory(D),
    File(F)
}

trait VFSMountPoint{
    fn build_index(&self) -> Index<HashMap<String, FileIndex>, FileIndex>;
}

slotmap::new_key_type!{
    pub struct MountPoint;
    pub struct FileIndex;
}

pub struct Path{
    pieces: Vec<usize>,
    s: String
} impl Path{
    pub fn from_string(input: &str) -> Path{
        let mut last = 0;
        let mut processed: Vec<&str> = Vec::new();
        let mut pieces: Vec<usize> = Vec::new();

        for slash in input.char_indices().filter_map(|(i,c)| if c == '/' {Some(i)} else {None}).chain(once(input.len())){
            let ss = &input[last..slash];
            if ss.is_empty(){
                // root slash
                processed.clear();
                pieces.clear();
            } else if ss == ".."{
                // Up one level
                processed.pop();
                pieces.pop();
            } else if ss == "."{
                // Do nothing, current dir
            } else {
                pieces.push(pieces.last().copied().unwrap_or(0) + ss.len());
                processed.push(ss);
            }
            last = slash;
        }
        let s = processed.concat();
        Path { pieces, s }
    }
    
    pub fn parent(&mut self) -> Option<String>{
        match self.pieces.len(){
            0 => {
                // Do nothing
                None
            },
            1 => {
                self.pieces.clear();
                Some(std::mem::replace(&mut self.s, String::new()))
            },
            n => {
                self.pieces.pop();

                // Saftey: the match checked for us
                Some(self.s.split_off(unsafe{*self.pieces.get_unchecked(n - 2)}))
            }
        }
    }

    pub fn name(&self) -> Option<&str>{
        match self.pieces.len(){
            0 => {
                // Do nothing
                None
            },
            1 => {
                Some(self.s.as_str())
            },
            n => {
                // Saftey: the match checked for us
                Some(&self.s[unsafe{*self.pieces.get_unchecked(n - 2)}..])
            }
        }
    }

    pub fn is_child_of(&self, other: &Path) -> bool{
        if other.s.len() > self.s.len(){return false}
        self.pieces.starts_with(&other.pieces) && self.s.starts_with(&other.s)
    }

    pub fn clear(&mut self){
        self.pieces.clear();
        self.s.clear();
    }
    
    pub fn join(&mut self, other: Path){
        let offset = self.s.len();
        self.pieces.extend(other.pieces.iter().map(|x| x + offset));
        self.s.push_str(other.s.as_str());
    }

    pub fn join_str(&mut self, s: &str){

    }
} impl Display for Path{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.pieces.is_empty(){
            f.write_char('/')
        } else{
            let mut ctr = 0;
            for p in self.pieces.iter().cloned(){
                f.write_char('/')?;
                f.write_str(&self.s[ctr..p])?;
                ctr = p;
            }
            
            Ok(())
        }
    }
}


pub struct VFS{
    mount_points: slotmap::SlotMap<MountPoint, (String, Box<dyn VFSMountPoint>)>,
    index: HashMap<String, Index<Vec<String>, FileIndex>>
} impl VFS{
    fn build_index(&mut self){
        for (base,mp) in self.mount_points.values(){
            
        }
    }
}