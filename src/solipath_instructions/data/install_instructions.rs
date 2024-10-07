use std::sync::Arc;

use crate::solipath_instructions::data::download_instruction::DownloadInstruction;
use crate::solipath_instructions::data::environment_variable::EnvironmentVariable;
use crate::solipath_instructions::data::install_command::InstallCommand;
use crate::solipath_instructions::data::template::Template;
use crate::solipath_platform::platform_filter::{filter_list, PlatformFilterTrait};
use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct InstallInstructions {
    #[serde(default = "default_downloads")]
    downloads: Vec<DownloadInstruction>,

    #[serde(default = "default_environment_variable")]
    environment_variables: Vec<EnvironmentVariable>,

    #[serde(default = "default_templates")]
    templates: Vec<Template>,

    #[serde(default = "default_commands")]
    install_commands: Vec<InstallCommand>,
}

fn default_downloads() -> Vec<DownloadInstruction> {
    Vec::new()
}

fn default_environment_variable() -> Vec<EnvironmentVariable> {
    Vec::new()
}

fn default_templates() -> Vec<Template> {
    Vec::new()
}

fn default_commands() -> Vec<InstallCommand> {
    Vec::new()
}

impl InstallInstructions {
    pub fn new(
        templates: Vec<Template>,
        downloads: Vec<DownloadInstruction>,
        environment_variables: Vec<EnvironmentVariable>,
        install_commands: Vec<InstallCommand>,
    ) -> Self {
        Self {
            templates,
            downloads,
            environment_variables,
            install_commands,
        }
    }
    pub fn get_downloads(&self) -> &Vec<DownloadInstruction> {
        &self.downloads
    }

    pub fn get_environment_variables(&self) -> &Vec<EnvironmentVariable> {
        &self.environment_variables
    }

    pub fn get_templates(&self) -> &Vec<Template> {
        &self.templates
    }

    pub fn get_install_commands(&self) -> &Vec<InstallCommand> {
        &self.install_commands
    }
    pub fn filter_platform(&self, platform_filter: &Arc<dyn PlatformFilterTrait>) -> Self {
        let templates = filter_list(platform_filter, self.get_templates());
        let downloads = filter_list(platform_filter, self.get_downloads());
        let environment_variables = filter_list(platform_filter, self.get_environment_variables());
        let install_commands = filter_list(platform_filter, self.get_install_commands());
        Self {
                templates,
                downloads,
                environment_variables,
                install_commands
        }
    }
}

#[cfg(test)]
mod tests{
    use std::sync::Arc;

    use crate::solipath_platform::{platform::Platform, platform_filter::{mock::FakeCurrentPlatformRetriever, PlatformFilter, PlatformFilterTrait}};

    use super::InstallInstructions;

    fn create_platform_filter()-> Arc<dyn PlatformFilterTrait> {
        let platform_retriever = FakeCurrentPlatformRetriever {
            platform: Platform::new("Matching OS", "Matching Arch"),
        };
        Arc::new(PlatformFilter::new(Arc::new(platform_retriever))) 
    }

    #[test]
    fn can_filter_environment_variables(){
        let platform_filter = create_platform_filter();
        let install_instructions = serde_json::from_str::<InstallInstructions>(
            r#"
            {"environment_variables": [
                {"name": "RUST_TEST", "relative_path": "some/path/location", "platform_filters": [{"os": "Matching OS", "arch": "Matching Arch"}]},
                {"name": "RUST_TEST", "relative_path": "some/path/location", "platform_filters": [{"os": "a bad match", "arch": "x86"}]}
            ]}"#).unwrap();

        let filtered_instructions = install_instructions.filter_platform(&platform_filter);
        assert_eq!(vec![install_instructions.environment_variables[0].clone()], filtered_instructions.environment_variables)
    }

    #[test]
    fn can_filter_downloads(){
        let platform_filter = create_platform_filter();
        let install_instructions = serde_json::from_str::<InstallInstructions>(
            r#"
            {"downloads": [
                {"url": "www.github.com/node15.zip", "destination_directory": "node15Folder", "platform_filters": [{"os": "Matching OS", "arch": "Matching Arch"}]},
                {"url": "www.github.com/gradle.zip", "destination_directory": "gradleFolder", "platform_filters": [{"os": "a bad match", "arch": "arm"}]}
            ]}"#).unwrap();

        let filtered_instructions = install_instructions.filter_platform(&platform_filter);
        assert_eq!(vec![install_instructions.downloads[0].clone()], filtered_instructions.downloads)
    }
    
    #[test]
    fn can_filter_install_commands(){
        let platform_filter = create_platform_filter();
        let install_instructions = serde_json::from_str::<InstallInstructions>(
            r#"{"install_commands": [
                {"command": "do something", "platform_filters": [{"os": "Matching OS", "arch": "Matching Arch"}]},
                {"command": "do something2", "platform_filters": [{"os": "a bad match", "arch": "x86"}]}
            ]}"#).unwrap();

        let filtered_instructions = install_instructions.filter_platform(&platform_filter);
        assert_eq!(vec![install_instructions.install_commands[0].clone()], filtered_instructions.install_commands)
    }
    #[test]
    fn can_filter_templates(){
        let platform_filter = create_platform_filter();
        let install_instructions = serde_json::from_str::<InstallInstructions>(
            r#"{"templates": [
                {"name": "template1", "variables": {"key1": "value1", "key2": "value2"}, "platform_filters": [{"os": "Matching OS", "arch": "Matching Arch"}]},
                {"name": "template2", "variables": {"key1": "value2", "key2": "value3"}, "platform_filters": [{"os": "a bad match", "arch": "x86"}]}
            ]}"#).unwrap();

        let filtered_instructions = install_instructions.filter_platform(&platform_filter);
        assert_eq!(vec![install_instructions.templates[0].clone()], filtered_instructions.templates)
    }
}