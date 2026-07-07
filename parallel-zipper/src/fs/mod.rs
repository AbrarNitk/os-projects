use crate::error::AResult;
use std::{
    fs::Metadata,
    path::{Path, PathBuf},
};

// at the root it can be a file/files or a directory
// if file need to store only the file name

#[derive(Debug)]
pub struct Files {
    // todo: contains only the directory name, instead of the path
    pub root: Option<PathBuf>,
    pub files: Vec<Metadata>,
}

pub fn read_and_collect(path: &Path) -> AResult<Files> {
    println!("path: {}", path.display());
    let mut collector = Vec::new();
    collect_util(path, &mut collector);

    let files = if path.is_dir() {
        Files {
            root: Some(path.to_path_buf()),
            files: collector,
        }
    } else {
        Files {
            root: None,
            files: collector,
        }
    };

    Ok(files)
}

pub fn collect_util(path: &Path, collector: &mut Vec<Metadata>) {
    if path.is_dir() {
        for entry in path.read_dir().expect("error in reading directory") {
            let entry = entry.expect("error in the reading the entry");
            let file_type = entry.file_type().expect("error in checking the file type");
            if file_type.is_dir() {
                collect_util(&entry.path(), collector);
            } else if file_type.is_file() {
                collector.push(entry.metadata().expect("error in collecting metadata"));
            } else {
                unimplemented!("supported only file and directory")
            }
        }
    } else if path.is_file() {
        collector.push(path.metadata().expect("error in collecting metadata"));
    } else {
        unimplemented!("supported only file and directory")
    }
}

#[cfg(test)]
mod test {
    use std::env::current_dir;

    #[test]
    fn test_me() {
        println!("{:?}", current_dir());
        let path = std::path::Path::new("../parallel-zipper");
        let files = super::read_and_collect(&path.canonicalize().expect("error in canoniclizing"));
        println!("paths: {:?}", files);
    }
}
