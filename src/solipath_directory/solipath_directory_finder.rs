use std::path::PathBuf;

use dirs_next::home_dir;

use crate::solipath_dependency_metadata::dependency::Dependency;
#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait SolipathDirectoryFinderTrait {
    fn get_base_solipath_directory(&self) -> PathBuf;
    fn get_dependency_version_directory(&self, dependency: &Dependency) -> PathBuf;
    fn get_dependency_downloads_directory(&self, dependency: &Dependency) -> PathBuf;
}

pub struct SolipathDirectoryFinder {}

impl SolipathDirectoryFinder {
    pub fn new() -> Self {
        Self {}
    }
}

impl SolipathDirectoryFinderTrait for SolipathDirectoryFinder {
    fn get_base_solipath_directory(&self) -> PathBuf {
        let mut solipath = home_dir().expect("failed to retrieve home dir");
        solipath.push("solipath");
        solipath
    }

    fn get_dependency_version_directory(&self, dependency: &Dependency) -> PathBuf {
        let mut path = self.get_base_solipath_directory();
        path.push(dependency.name.to_string());
        path.push(dependency.version.to_string());
        path
    }

    fn get_dependency_downloads_directory(&self, dependency: &Dependency) -> PathBuf {
        let mut path = self.get_base_solipath_directory();
        path.push(dependency.name.to_string());
        path.push("downloads");
        path
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn base_solipath_directory_ends_with_solipath() {
        let solipath_directory = SolipathDirectoryFinder::new().get_base_solipath_directory();
        let home_dir = home_dir().unwrap().to_str().unwrap().to_string();
        assert_eq!(solipath_directory, PathBuf::from(format!("{}/solipath", home_dir)));
    }

    #[test]
    fn dependency_solipath_directory_ends_with_solipath_dependency_name_version() {
        let dependency = Dependency::new("java", "11");
        let solipath_directory = SolipathDirectoryFinder::new().get_dependency_version_directory(&dependency);
        let home_dir = home_dir().unwrap().to_str().unwrap().to_string();
        assert_eq!(
            solipath_directory,
            PathBuf::from(format!("{}/solipath/java/11", home_dir))
        );
    }

    #[test]
    fn dependency_downloads_solipath_directory_ends_with_solipath_dependency_name() {
        let dependency = Dependency::new("node", "14");
        let solipath_directory = SolipathDirectoryFinder::new().get_dependency_downloads_directory(&dependency);
        let home_dir = home_dir().unwrap().to_str().unwrap().to_string();
        assert_eq!(
            solipath_directory,
            PathBuf::from(format!("{}/solipath/node/downloads", home_dir))
        );
    }
}
