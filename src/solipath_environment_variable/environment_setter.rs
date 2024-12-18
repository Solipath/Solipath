use crate::solipath_instructions::data::dependency::Dependency;
use crate::solipath_directory::solipath_directory_finder::SolipathDirectoryFinderTrait;
use crate::solipath_instructions::data::environment_variable::EnvironmentVariable;
use std::env::join_paths;
use std::env::set_var;
use std::env::split_paths;
use std::env::var;
use std::path::PathBuf;
use std::sync::Arc;

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait EnvironmentSetterTrait {
    fn set_variable(&self, dependency: &Dependency, environment_variable: &EnvironmentVariable);
}
pub struct EnvironmentSetter {
    directory_finder: Arc<dyn SolipathDirectoryFinderTrait + Sync + Send>,
}

impl EnvironmentSetter {
    pub fn new(directory_finder: Arc<dyn SolipathDirectoryFinderTrait + Sync + Send>) -> Self {
        Self { directory_finder }
    }

    fn convert_relative_path_from_downloads_directory_to_absolute_path(
        &self,
        dependency: &Dependency,
        environment_variable: &EnvironmentVariable,
    ) -> PathBuf {
        let mut download_directory = self.directory_finder.get_dependency_downloads_directory(&dependency);
        download_directory.push(environment_variable.get_relative_path().as_ref().expect("expected relative path to be defined"));
        download_directory
    }
}

impl EnvironmentSetterTrait for EnvironmentSetter {
    fn set_variable(&self, dependency: &Dependency, environment_variable: &EnvironmentVariable) {
        let name = environment_variable.get_name();
        if name == "PATH" {
            let path_value = if let Some(value) = environment_variable.get_value() {
                PathBuf::from(value.clone())
            } else {
                self.convert_relative_path_from_downloads_directory_to_absolute_path(dependency, environment_variable)
            };
            append_to_path(path_value);
        } else {
            if let Some(value) = environment_variable.get_value(){
                set_var(name, value)
            } else {
                set_var(name, self.convert_relative_path_from_downloads_directory_to_absolute_path(dependency, environment_variable));
            }
        }
    }
}

fn append_to_path(absolute_path: PathBuf) {
    let mut split_paths = split_paths(&var("PATH").expect("failed to get PATH variable")).collect::<Vec<_>>();
    let mut path = vec![absolute_path];
    path.append(&mut split_paths);
    let combined_path = join_paths(path).expect("failed to combine paths");
    set_var("PATH", combined_path);
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::solipath_directory::solipath_directory_finder::MockSolipathDirectoryFinderTrait;
    use mockall::predicate::*;

    #[test]
    fn can_set_variable_for_rust_test() {
        let dependency = Dependency::new("dependency", "123.12");
        let environment_variable = serde_json::from_str::<EnvironmentVariable>(
            r#"{"name": "RUST_TEST", "relative_path": "some/path/location"}"#,
        )
        .unwrap();
        let mut directory_finder = MockSolipathDirectoryFinderTrait::new();
        directory_finder
            .expect_get_dependency_downloads_directory()
            .with(eq(dependency.clone()))
            .return_const(PathBuf::from("solipath/home/downloads/dir"));
        let environment_setter = EnvironmentSetter::new(Arc::new(directory_finder));
        environment_setter.set_variable(&dependency, &environment_variable);
        assert_eq!(
            PathBuf::from(var("RUST_TEST").unwrap()),
            PathBuf::from("solipath/home/downloads/dir/some/path/location")
        );
    }

    #[test]
    fn can_set_variable_as_value() {
        let dependency = Dependency::new("dependency", "123.12");
        let environment_variable = serde_json::from_str::<EnvironmentVariable>(
            r#"{"name": "RUST_TEST", "value": "someValue"}"#,
        )
        .unwrap();
        let directory_finder = MockSolipathDirectoryFinderTrait::new();
        let environment_setter = EnvironmentSetter::new(Arc::new(directory_finder));
        environment_setter.set_variable(&dependency, &environment_variable);
        assert_eq!(
            PathBuf::from(var("RUST_TEST").unwrap()),
            PathBuf::from("someValue")
        );
    }
    #[test]
    fn can_append_value_to_path() {
        let original_path = var("PATH").unwrap();
        let dependency = Dependency::new("dependency", "555.213");
        let environment_variable =
            serde_json::from_str::<EnvironmentVariable>(r#"{"name": "PATH", "value": "~/path/location"}"#)
                .unwrap();
        let directory_finder = MockSolipathDirectoryFinderTrait::new();
        let environment_setter = EnvironmentSetter::new(Arc::new(directory_finder));
        environment_setter.set_variable(&dependency, &environment_variable);
        let expected_path = PathBuf::from("~/path/location");
        assert!(var("PATH").unwrap().starts_with(expected_path.to_str().unwrap()));
        assert!(var("PATH").unwrap().ends_with(&original_path));
    }
    

    #[test]
    fn can_append_to_path() {
        let original_path = var("PATH").unwrap();
        let dependency = Dependency::new("dependency", "555.213");
        let environment_variable =
            serde_json::from_str::<EnvironmentVariable>(r#"{"name": "PATH", "relative_path": "some/path/location"}"#)
                .unwrap();
        let mut directory_finder = MockSolipathDirectoryFinderTrait::new();
        directory_finder
            .expect_get_dependency_downloads_directory()
            .with(eq(dependency.clone()))
            .return_const(PathBuf::from("solipath/home/downloads"));
        let environment_setter = EnvironmentSetter::new(Arc::new(directory_finder));
        environment_setter.set_variable(&dependency, &environment_variable);
        let mut expected_path = PathBuf::from("solipath/home/downloads");
        expected_path.push("some/path/location");
        assert!(var("PATH").unwrap().starts_with(expected_path.to_str().unwrap()));
        assert!(var("PATH").unwrap().ends_with(&original_path));
    }
}
