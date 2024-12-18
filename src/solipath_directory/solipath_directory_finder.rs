use std::path::PathBuf;

use directories::UserDirs;

use crate::solipath_instructions::data::dependency::Dependency;

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait SolipathDirectoryFinderTrait {
    fn get_base_solipath_directory(&self) -> PathBuf;

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

    fn get_dependency_template_directory(&self, dependency: &Dependency) -> PathBuf {
        let mut path = self.get_base_solipath_directory();
        path.push(dependency.name.to_string());
        path.push("templates");
        path
    }
}

fn home_dir()-> PathBuf{
    UserDirs::new().unwrap().home_dir().to_path_buf()
}

pub struct SolipathDirectoryFinder {}

impl SolipathDirectoryFinder {
    pub fn new() -> Self {
        Self {}
    }
}

impl SolipathDirectoryFinderTrait for SolipathDirectoryFinder {
    fn get_base_solipath_directory(&self) -> PathBuf {
        let mut solipath = home_dir();
        solipath.push("solipath");
        solipath
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn base_solipath_directory_ends_with_solipath() {
        let solipath_directory = SolipathDirectoryFinder::new().get_base_solipath_directory();
        let home_dir = home_dir().to_str().unwrap().to_string();
        assert_eq!(solipath_directory, PathBuf::from(format!("{}/solipath", home_dir)));
    }

    #[test]
    fn dependency_solipath_directory_ends_with_solipath_dependency_name_version() {
        let dependency = Dependency::new("java", "11");
        let solipath_directory = SolipathDirectoryFinder::new().get_dependency_version_directory(&dependency);
        let home_dir = home_dir().to_str().unwrap().to_string();
        assert_eq!(
            solipath_directory,
            PathBuf::from(format!("{}/solipath/java/11", home_dir))
        );
    }

    #[test]
    fn dependency_downloads_solipath_directory_ends_with_solipath_dependency_name() {
        let dependency = Dependency::new("node", "14");
        let solipath_directory = SolipathDirectoryFinder::new().get_dependency_downloads_directory(&dependency);
        let home_dir = home_dir().to_str().unwrap().to_string();
        assert_eq!(
            solipath_directory,
            PathBuf::from(format!("{}/solipath/node/downloads", home_dir))
        );
    }

    #[test]
    fn dependency_template_solipath_directory_ends_with_solipath_dependency_name() {
        let dependency = Dependency::new("node", "14");
        let solipath_directory = SolipathDirectoryFinder::new().get_dependency_template_directory(&dependency);
        let home_dir = home_dir().to_str().unwrap().to_string();
        assert_eq!(
            solipath_directory,
            PathBuf::from(format!("{}/solipath/node/templates", home_dir))
        );
    }
}
