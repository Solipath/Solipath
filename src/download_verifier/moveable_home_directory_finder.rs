use std::path::PathBuf;
use solipath_lib::solipath_directory::solipath_directory_finder::SolipathDirectoryFinderTrait;

pub struct MoveableHomeDirectoryFinder {
    base_dir: PathBuf,
}

impl MoveableHomeDirectoryFinder {
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }
}

impl SolipathDirectoryFinderTrait for MoveableHomeDirectoryFinder {
    fn get_base_solipath_directory(&self) -> PathBuf {
        self.base_dir.clone()
    }
}