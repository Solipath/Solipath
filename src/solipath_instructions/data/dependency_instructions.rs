use crate::solipath_dependency_metadata::dependency::Dependency;
use crate::solipath_instructions::data::template::Template;
use crate::solipath_instructions::data::{
    download_instruction::DownloadInstruction, environment_variable::EnvironmentVariable,
    install_instructions::InstallInstructions,
    install_command::InstallCommand
};
use crate::solipath_platform::platform::Platform;
use crate::solipath_platform::platform_filter::HasPlatformFilter;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DependencyInstructions {
    dependency: Dependency,
    install_instructions: InstallInstructions,
}



impl DependencyInstructions {
    pub fn new(dependency: Dependency, install_instructions: InstallInstructions) -> Self {
        Self {
            dependency,
            install_instructions,
        }
    }
    pub fn get_dependency(&self) -> &Dependency {
        &self.dependency
    }

    pub fn get_environment_variables(&self) -> &Vec<EnvironmentVariable> {
        self.install_instructions.get_environment_variables()
    }
    pub fn get_downloads(&self) -> &Vec<DownloadInstruction> {
        self.install_instructions.get_downloads()
    }
    pub fn get_templates(&self) -> &Vec<Template> {
        self.install_instructions.get_templates()
    }
    pub fn get_install_commands(&self) -> &Vec<InstallCommand> {
        self.install_instructions.get_install_commands()
    }
}

pub trait VecDependencyInstructions {
    fn get_environment_variables(&self) -> Vec<(&Dependency,&EnvironmentVariable)>;
    fn get_downloads(&self) -> Vec<(&Dependency, &DownloadInstruction)>;
}

impl VecDependencyInstructions for Vec<DependencyInstructions> {
    fn get_environment_variables(&self) -> Vec<(&Dependency, &EnvironmentVariable)> {
        self.iter().map(|instructions| 
            instructions.get_environment_variables().iter().map(|environment_variable|
                (instructions.get_dependency(), environment_variable))
    ).flatten().collect()
    }
    fn get_downloads(&self) -> Vec<(&Dependency, &DownloadInstruction)> {
        self.iter().map(|instructions| 
            instructions.get_downloads().iter().map(|download|
                (instructions.get_dependency(), download))
    ).flatten().collect()
    }
}

impl <T> HasPlatformFilter for (&Dependency, &T) where T: HasPlatformFilter{
    fn get_platform_filters(&self) -> &[Platform] {
        self.1.get_platform_filters()
    }
}

#[cfg(test)]
mod test {
    use crate::{solipath_dependency_metadata::dependency::Dependency, solipath_instructions::data::{dependency_instructions::VecDependencyInstructions, install_instructions::InstallInstructions}};

    use super::DependencyInstructions;

    #[test]
    fn can_get_aggregated_list_of_environment_variables() {
        let environment_variable_json = r#"
        {"environment_variables": [
            {"name": "RUST_TEST", "relative_path": "some/path/location", "platform_filters": [{"os": "a good match", "arch": "x86"}]},
            {"name": "RUST_TEST", "relative_path": "some/path/location", "platform_filters": [{"os": "a bad match", "arch": "x86"}]}
        ]}"#;
        let environment_variable_json2 = r#"
        {"environment_variables": [
            {"name": "RUST_TEST2", "relative_path": "some/path/location", "platform_filters": [{"os": "another match", "arch": "x86"}]}
        ]}"#;
        let dependency_instructions = vec![
            DependencyInstructions::new(
                Dependency::new("Dependency1", "1.0"), serde_json::from_str::<InstallInstructions>(environment_variable_json).unwrap()
            ),
            DependencyInstructions::new(
                Dependency::new("Dependency2", "2.0"), serde_json::from_str::<InstallInstructions>(environment_variable_json2).unwrap()
            )
        ];
        assert_eq!(vec![
                (dependency_instructions[0].get_dependency(), &dependency_instructions[0].get_environment_variables()[0]),
                (dependency_instructions[0].get_dependency(), &dependency_instructions[0].get_environment_variables()[1]),
                (dependency_instructions[1].get_dependency(), &dependency_instructions[1].get_environment_variables()[0])
            ],
            dependency_instructions.get_environment_variables())
    }
    
    #[test]
    fn can_get_aggregated_list_of_downloads() {
        let downloads_json = r#"
        {"downloads": [
            {"url": "www.github.com/node15.zip", "destination_directory": "node15Folder", "platform_filters": [{"os": "a good match", "arch": "x86"}]},
            {"url": "www.github.com/gradle.zip", "destination_directory": "gradleFolder", "platform_filters": [{"os": "a bad match", "arch": "arm"}]}
        ]}"#;
        let downloads_json2 = r#"
        {"downloads": [
            {"url": "www.github.com/node15.zip", "destination_directory": "node15Folder", "platform_filters": [{"os": "a good match", "arch": "x86"}]}
        ]}"#;
        let dependency_instructions = vec![
            DependencyInstructions::new(
                Dependency::new("Dependency1", "1.0"), serde_json::from_str::<InstallInstructions>(downloads_json).unwrap()
            ),
            DependencyInstructions::new(
                Dependency::new("Dependency2", "2.0"), serde_json::from_str::<InstallInstructions>(downloads_json2).unwrap()
            )
        ];
        assert_eq!(vec![
                (dependency_instructions[0].get_dependency(), &dependency_instructions[0].get_downloads()[0]),
                (dependency_instructions[0].get_dependency(), &dependency_instructions[0].get_downloads()[1]),
                (dependency_instructions[1].get_dependency(), &dependency_instructions[1].get_downloads()[0])
            ],
            dependency_instructions.get_downloads())
    }
}