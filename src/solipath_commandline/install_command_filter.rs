#[cfg(test)]
use mockall::automock;

use std::{collections::HashMap, sync::Arc};

use crate::{solipath_dependency_metadata::dependency::Dependency, solipath_directory::solipath_directory_finder::SolipathDirectoryFinderTrait};

#[cfg_attr(test, automock)]
pub trait InstallCommandFilterTrait{
    fn command_should_be_run(&self, dependency: &Dependency, when_to_run_rules: &HashMap<String,  serde_json::Value>)-> bool;
}

pub struct InstallCommandFilter{
    directory_finder: Arc<dyn SolipathDirectoryFinderTrait + Sync + Send>
}
impl InstallCommandFilter{
    pub fn new(directory_finder: Arc<dyn SolipathDirectoryFinderTrait + Sync + Send>)-> Self{
        Self{directory_finder}
    }
    fn check_single_rule(&self, dependency: &Dependency, rule_name: &String, rule_value: &serde_json::Value)-> bool {
        match rule_name.as_str() {
            "file_does_not_exist" => self.check_file_does_not_exist(dependency, rule_value.as_str().expect("file_does_exist should be a string")),
            _ => panic!("unrecognized command filter option '{}' in '{}' dependency!", rule_name, dependency.name)
        }
    }
    fn check_file_does_not_exist(&self, dependency: &Dependency, relative_file_path: &str)-> bool {
        let mut downloads_path = self.directory_finder.get_dependency_downloads_directory(dependency);
        downloads_path.push(relative_file_path);
        !downloads_path.exists()
    }
}

impl InstallCommandFilterTrait for InstallCommandFilter{
    fn command_should_be_run(&self, dependency: &Dependency, when_to_run_rules: &HashMap<String,  serde_json::Value>)-> bool {
        when_to_run_rules.iter().fold(true, |should_run, (key, value)|{
            should_run & self.check_single_rule(dependency, key, value)
        })
    }
}


#[cfg(test)]
mod tests{
    use std::fs::File;

    use mockall::predicate::eq;
    use tempfile::tempdir;

    use super::*;
    use crate::{solipath_dependency_metadata::dependency::Dependency, solipath_directory::solipath_directory_finder::MockSolipathDirectoryFinderTrait};

    #[test]
    fn filter_commands_no_rules_returns_true() {
        let directory_finder = MockSolipathDirectoryFinderTrait::new();
        let dependency = Dependency::new("depend", "version");
        let install_command_filter = InstallCommandFilter::new(Arc::new(directory_finder));
        assert_eq!(install_command_filter.command_should_be_run(&dependency, &HashMap::new()), true);
    }

    #[test]
    fn filter_commands_file_exists_already_on_a_file_does_not_exist_rule_returns_false() {
        let mut directory_finder = MockSolipathDirectoryFinderTrait::new();
        let dependency = Dependency::new("depend", "version");
        let temp_dir = tempdir().unwrap().into_path();
        directory_finder.expect_get_dependency_downloads_directory()
            .with(eq(dependency.clone()))
            .return_const(temp_dir.clone());
        let expected_file_path = temp_dir.clone().join("somepath".to_string());
        File::create(expected_file_path).expect("failed to create file");
        let mut map = HashMap::new();
        map.insert("file_does_not_exist".to_string(), serde_json::Value::String("somepath".to_string()));
        let install_command_filter = InstallCommandFilter::new(Arc::new(directory_finder));
        assert_eq!(install_command_filter.command_should_be_run(&dependency, &map), false);
    }

    #[test]
    fn filter_commands_file_does_not_exist_on_a_file_does_not_exist_rule_returns_true() {
        let mut directory_finder = MockSolipathDirectoryFinderTrait::new();
        let dependency = Dependency::new("depend", "version");
        let temp_dir = tempdir().unwrap().into_path();
        directory_finder.expect_get_dependency_downloads_directory()
            .with(eq(dependency.clone()))
            .return_const(temp_dir.clone());
        let mut map = HashMap::new();
        map.insert("file_does_not_exist".to_string(), serde_json::Value::String("somepath".to_string()));
        let install_command_filter = InstallCommandFilter::new(Arc::new(directory_finder));
        assert_eq!(install_command_filter.command_should_be_run(&dependency, &map), true);
    }

    #[test]
    #[should_panic(expected="unrecognized command filter option 'nonexistent_filter' in 'depend' dependency!")]
    fn filter_commands_unrecognized_filter_panics() {
        let directory_finder = MockSolipathDirectoryFinderTrait::new();
        let dependency = Dependency::new("depend", "version");
        let mut map = HashMap::new();
        map.insert("nonexistent_filter".to_string(), serde_json::Value::String("something".to_string()));
        let install_command_filter = InstallCommandFilter::new(Arc::new(directory_finder));
        install_command_filter.command_should_be_run(&dependency, &map);
    }    

}