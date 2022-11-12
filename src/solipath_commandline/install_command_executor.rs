#[cfg(test)]
use mockall::automock;

use std::fs::create_dir_all;
use std::path::PathBuf;
use std::sync::Arc;

use crate::{solipath_commandline::command_executor::CommandExecutorTrait, solipath_dependency_metadata::dependency::Dependency, solipath_directory::solipath_directory_finder::SolipathDirectoryFinderTrait, solipath_instructions::data::install_command::InstallCommand};
use crate::solipath_commandline::install_command_filter::InstallCommandFilterTrait;

#[cfg_attr(test, automock)]
pub trait InstallCommandExecutorTrait{
    fn execute_command(&self, dependency: &Dependency, install_command: &InstallCommand);
}

pub struct InstallCommandExecutor{
    command_executor: Arc<dyn CommandExecutorTrait + Sync + Send>,
    install_command_filter: Arc<dyn InstallCommandFilterTrait + Sync + Send>,
    directory_finder: Arc<dyn SolipathDirectoryFinderTrait + Sync + Send>
}

impl InstallCommandExecutor {
    pub fn new(
        command_executor: Arc<dyn CommandExecutorTrait + Sync + Send>, 
        install_command_filter: Arc<dyn InstallCommandFilterTrait + Sync + Send>,
        directory_finder: Arc<dyn SolipathDirectoryFinderTrait + Sync + Send>
    )-> Self{
        Self{command_executor, install_command_filter, directory_finder}
    }
}

impl InstallCommandExecutorTrait for InstallCommandExecutor {
    fn execute_command(&self, dependency: &Dependency, install_command: &InstallCommand) {
        if self.install_command_filter.command_should_be_run(dependency, &install_command.get_when_to_run_rules()) {
            let downloads_directory = self.directory_finder.get_dependency_downloads_directory(dependency);
            create_dir_all(&downloads_directory).expect("failed to create downloads directory");

            let command_string = format!("{} && {}", switch_to_download_directory_command(&downloads_directory), install_command.get_command());
            self.command_executor.execute_single_string_command(command_string);
        }
    }
}

fn switch_to_download_directory_command(downloads_directory: &PathBuf)-> String {
    if std::env::consts::OS == "windows" {
        format!("cd /d {}", downloads_directory.to_str().unwrap())
    } else {
        format!("cd {}", downloads_directory.to_str().unwrap())
    }
}


#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use mockall::predicate::eq;

    use super::*;
    use crate::solipath_commandline::command_executor::MockCommandExecutorTrait;
    use crate::solipath_directory::solipath_directory_finder::MockSolipathDirectoryFinderTrait;
    use crate::solipath_commandline::install_command_filter::MockInstallCommandFilterTrait;


    #[test]
    fn run_command_if_rules_pass(){
        let dependency = Dependency::new("depend", "version");
        let mut command_rules = HashMap::new();
        command_rules.insert("file_does_not_exist".to_string(), serde_json::Value::String("thefile".to_string()));
        let mut command_filter = MockInstallCommandFilterTrait::new();
        command_filter.expect_command_should_be_run()
            .with(eq(dependency.clone()), eq(command_rules))
            .return_const(true);
        let mut directory_finder = MockSolipathDirectoryFinderTrait::new();
        directory_finder.expect_get_dependency_downloads_directory()
            .with(eq(dependency.clone()))
            .return_const("downloads_directory");
        let mut command_executor = MockCommandExecutorTrait::new();
        
        let command_expectation = command_executor.expect_execute_single_string_command();
        if std::env::consts::OS == "windows" {
            command_expectation.with(eq("cd /d downloads_directory && do something".to_string()))
            .return_const(());
        } else {
            command_expectation.with(eq("cd downloads_directory && do something".to_string()))
            .return_const(());
        }
        let install_command_executor = InstallCommandExecutor::new(
            Arc::new(command_executor),
            Arc::new(command_filter),
            Arc::new(directory_finder)
        );
        let install_command: InstallCommand = serde_json::from_str(r#"{
            "command": "do something", 
            "when_to_run_rules": {"file_does_not_exist": "thefile"}}
        "#).expect("failed to parse string");
        install_command_executor.execute_command(&dependency, &install_command);
    }

    #[test]
    fn do_not_run_command_if_rules_fail(){
        let dependency = Dependency::new("depend", "version");
        let mut command_rules = HashMap::new();
        command_rules.insert("file_does_not_exist".to_string(), serde_json::Value::String("thefile".to_string()));
        let mut command_filter = MockInstallCommandFilterTrait::new();
        command_filter.expect_command_should_be_run()
            .with(eq(dependency.clone()), eq(command_rules))
            .return_const(false);
        let directory_finder = MockSolipathDirectoryFinderTrait::new();
        let command_executor = MockCommandExecutorTrait::new();    

        let install_command_executor = InstallCommandExecutor::new(
            Arc::new(command_executor),
            Arc::new(command_filter),
            Arc::new(directory_finder)
        );
        let install_command: InstallCommand = serde_json::from_str(r#"{
            "command": "do something", 
            "when_to_run_rules": {"file_does_not_exist": "thefile"}}
        "#).expect("failed to parse string");
        install_command_executor.execute_command(&dependency, &install_command);
    }
}