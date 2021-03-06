use crate::solipath_dependency_metadata::dependency::Dependency;
use crate::solipath_instructions::data::template::Template;
use crate::solipath_instructions::data::{
    download_instruction::DownloadInstruction, environment_variable::EnvironmentVariable,
    install_instructions::InstallInstructions,
};

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
    pub fn get_dependency(&self) -> Dependency {
        self.dependency.clone()
    }

    pub fn get_environment_variables(&self) -> Vec<EnvironmentVariable> {
        self.install_instructions.get_environment_variables()
    }
    pub fn get_downloads(&self) -> Vec<DownloadInstruction> {
        self.install_instructions.get_downloads()
    }
    pub fn get_templates(&self) -> Vec<Template> {
        self.install_instructions.get_templates()
    }
}
