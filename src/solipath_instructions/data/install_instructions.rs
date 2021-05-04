use crate::solipath_instructions::data::download_instruction::DownloadInstruction;
use crate::solipath_instructions::data::environment_variable::EnvironmentVariable;
use crate::solipath_instructions::data::install_command::InstallCommand;
use crate::solipath_instructions::data::template::Template;
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
    pub fn get_downloads(&self) -> Vec<DownloadInstruction> {
        self.downloads.clone()
    }

    pub fn get_environment_variables(&self) -> Vec<EnvironmentVariable> {
        self.environment_variables.clone()
    }

    pub fn get_templates(&self) -> Vec<Template> {
        self.templates.clone()
    }

    pub fn get_install_commands(&self) -> Vec<InstallCommand> {
        self.install_commands.clone()
    }
}
