use std::sync::Arc;

use serde::Deserialize;

use crate::solipath_instructions::data::dependency::Dependency;
use crate::solipath_instructions::data::template::Template;
use crate::solipath_instructions::data::{
    download_instruction::DownloadInstruction, environment_variable::EnvironmentVariable,
    install_command::InstallCommand, install_instructions::InstallInstructions,
};
use crate::solipath_platform::platform::Platform;
use crate::solipath_platform::platform_filter::{HasPlatformFilter, PlatformFilterTrait};


#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
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

    pub fn filter_platform(&self, platform_filter: &Arc<dyn PlatformFilterTrait>) -> Self {
        Self {
            dependency: self.dependency.clone(),
            install_instructions: self.install_instructions.filter_platform(platform_filter),
        }
    }
}

pub trait VecDependencyInstructions {
    fn get_environment_variables(&self) -> Vec<(&Dependency, &EnvironmentVariable)>;
    fn get_downloads(&self) -> Vec<(&Dependency, &DownloadInstruction)>;
    fn get_install_commands(&self) -> Vec<(&Dependency, &InstallCommand)>;
    fn get_templates(&self) -> Vec<(&Dependency, &Template)>;
    fn filter_platform(&self, platform_filter: &Arc<dyn PlatformFilterTrait>) -> Self;
}

impl VecDependencyInstructions for Vec<DependencyInstructions> {
    fn get_environment_variables(&self) -> Vec<(&Dependency, &EnvironmentVariable)> {
        group_dependency_with_field(self, |instructions| instructions.get_environment_variables())
    }
    fn get_downloads(&self) -> Vec<(&Dependency, &DownloadInstruction)> {
        group_dependency_with_field(self, |instructions| instructions.get_downloads())
    }
    fn get_install_commands(&self) -> Vec<(&Dependency, &InstallCommand)> {
        group_dependency_with_field(self, |instructions| instructions.get_install_commands())
    }
    fn get_templates(&self) -> Vec<(&Dependency, &Template)> {
        group_dependency_with_field(self, |instructions| instructions.get_templates())
    }

    fn filter_platform(&self, platform_filter: &Arc<dyn PlatformFilterTrait>)-> Self{
        self.iter()
            .map(|dependency_instructions| dependency_instructions.filter_platform(platform_filter))
            .collect()
    }
}

fn group_dependency_with_field<'a, FUNCTION, FIELD>(
    list: &'a Vec<DependencyInstructions>,
    get_function: FUNCTION,
) -> Vec<(&'a Dependency, &'a FIELD)>
where
    FUNCTION: Fn(&'a DependencyInstructions) -> &'a Vec<FIELD>,
{
    list.iter()
        .map(|instructions| {
            get_function(instructions)
                .iter()
                .map(|environment_variable| (instructions.get_dependency(), environment_variable))
        })
        .flatten()
        .collect()
}

impl<T> HasPlatformFilter for (&Dependency, &T)
where
    T: HasPlatformFilter,
{
    fn get_platform_filters(&self) -> &[Platform] {
        self.1.get_platform_filters()
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use crate::{
        solipath_instructions::data::{
            dependency_instructions::VecDependencyInstructions, install_instructions::InstallInstructions,
        }, solipath_platform::{platform::Platform, platform_filter::{mock::FakeCurrentPlatformRetriever, PlatformFilter, PlatformFilterTrait}},
    };

    use super::*;

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
                Dependency::new("Dependency1", "1.0"),
                serde_json::from_str::<InstallInstructions>(environment_variable_json).unwrap(),
            ),
            DependencyInstructions::new(
                Dependency::new("Dependency2", "2.0"),
                serde_json::from_str::<InstallInstructions>(environment_variable_json2).unwrap(),
            ),
        ];
        assert_eq!(
            vec![
                (
                    dependency_instructions[0].get_dependency(),
                    &dependency_instructions[0].get_environment_variables()[0]
                ),
                (
                    dependency_instructions[0].get_dependency(),
                    &dependency_instructions[0].get_environment_variables()[1]
                ),
                (
                    dependency_instructions[1].get_dependency(),
                    &dependency_instructions[1].get_environment_variables()[0]
                )
            ],
            dependency_instructions.get_environment_variables()
        )
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
                Dependency::new("Dependency1", "1.0"),
                serde_json::from_str::<InstallInstructions>(downloads_json).unwrap(),
            ),
            DependencyInstructions::new(
                Dependency::new("Dependency2", "2.0"),
                serde_json::from_str::<InstallInstructions>(downloads_json2).unwrap(),
            ),
        ];
        assert_eq!(
            vec![
                (
                    dependency_instructions[0].get_dependency(),
                    &dependency_instructions[0].get_downloads()[0]
                ),
                (
                    dependency_instructions[0].get_dependency(),
                    &dependency_instructions[0].get_downloads()[1]
                ),
                (
                    dependency_instructions[1].get_dependency(),
                    &dependency_instructions[1].get_downloads()[0]
                )
            ],
            dependency_instructions.get_downloads()
        )
    }

    #[test]
    fn can_get_aggregated_install_commands() {
        let install_command_json = r#"{"install_commands": [
            {"command": "do something", "platform_filters": [{"os": "a good match", "arch": "x86"}]},
            {"command": "do something2", "platform_filters": [{"os": "a bad match", "arch": "x86"}]}
        ]}"#;
        let install_command_json2 = r#"
        {"install_commands": [
            {"command": "do something2", "platform_filters": [{"os": "a bad match", "arch": "x86"}]}
        ]}"#;
        let dependency_instructions = vec![
            DependencyInstructions::new(
                Dependency::new("Dependency1", "1.0"),
                serde_json::from_str::<InstallInstructions>(install_command_json).unwrap(),
            ),
            DependencyInstructions::new(
                Dependency::new("Dependency2", "2.0"),
                serde_json::from_str::<InstallInstructions>(install_command_json2).unwrap(),
            ),
        ];
        assert_eq!(
            vec![
                (
                    dependency_instructions[0].get_dependency(),
                    &dependency_instructions[0].get_install_commands()[0]
                ),
                (
                    dependency_instructions[0].get_dependency(),
                    &dependency_instructions[0].get_install_commands()[1]
                ),
                (
                    dependency_instructions[1].get_dependency(),
                    &dependency_instructions[1].get_install_commands()[0]
                )
            ],
            dependency_instructions.get_install_commands()
        )
    }

    #[test]
    fn can_get_aggregated_templates() {
        let install_instructions_json = r#"{"templates": [
                {"name": "template1", "variables": {"key1": "value1", "key2": "value2"}},
                {"name": "template2", "variables": {"key1": "value2", "key2": "value3"}}
            ]}"#;
        let install_instructions_json2 = r#"{"templates": [
                {"name": "template1", "variables": {"key1": "value1", "key2": "value2"}}
            ]}"#;
        let dependency_instructions = vec![
            DependencyInstructions::new(
                Dependency::new("Dependency1", "1.0"),
                serde_json::from_str::<InstallInstructions>(install_instructions_json).unwrap(),
            ),
            DependencyInstructions::new(
                Dependency::new("Dependency2", "2.0"),
                serde_json::from_str::<InstallInstructions>(install_instructions_json2).unwrap(),
            ),
        ];
        assert_eq!(
            vec![
                (
                    dependency_instructions[0].get_dependency(),
                    &dependency_instructions[0].get_templates()[0]
                ),
                (
                    dependency_instructions[0].get_dependency(),
                    &dependency_instructions[0].get_templates()[1]
                ),
                (
                    dependency_instructions[1].get_dependency(),
                    &dependency_instructions[1].get_templates()[0]
                )
            ],
            dependency_instructions.get_templates()
        )
    }

    fn create_platform_filter()-> Arc<dyn PlatformFilterTrait> {
        let platform_retriever = FakeCurrentPlatformRetriever {
            platform: Platform::new("Matching OS", "Matching Arch"),
        };
        Arc::new(PlatformFilter::new(Arc::new(platform_retriever))) 
    }

    #[test]
    fn can_filter_instructions_list() {
        let platform_filter = create_platform_filter();
        let install_instructions_json = r#"{"templates": [
                {"name": "template1", "variables": {"key1": "value1", "key2": "value2"}, "platform_filters": [{"os": "Matching OS", "arch": "Matching Arch"}]},
                {"name": "template2", "variables": {"key1": "value2", "key2": "value3"}, "platform_filters": [{"os": "Non Matching OS", "arch": "Non Matching Arch"}]}
            ]}"#;
        let install_instructions_json2 = r#"{"templates": [
                {"name": "template3", "variables": {"key1": "value1", "key2": "value2"}}
            ]}"#;
        let dependency_instructions = vec![
            DependencyInstructions::new(
                Dependency::new("Dependency1", "1.0"),
                serde_json::from_str::<InstallInstructions>(install_instructions_json).unwrap(),
            ),
            DependencyInstructions::new(
                Dependency::new("Dependency2", "2.0"),
                serde_json::from_str::<InstallInstructions>(install_instructions_json2).unwrap(),
            ),
        ];
        assert_eq!(
            serde_json::from_str::<Vec<DependencyInstructions>>(r#"[
                {
                    "dependency": {"name":"Dependency1", "version":"1.0"},
                    "install_instructions": {"templates": [
                        {"name": "template1", "variables": {"key1": "value1", "key2": "value2"}, "platform_filters": [{"os": "Matching OS", "arch": "Matching Arch"}]}
                    ]}
                },
                {
                    "dependency": {"name":"Dependency2", "version":"2.0"},
                    "install_instructions": {"templates": [
                        {"name": "template3", "variables": {"key1": "value1", "key2": "value2"}}
                    ]}
                }
            ]
            "#).unwrap(),
            dependency_instructions.filter_platform(&platform_filter)
        );
    }
}
