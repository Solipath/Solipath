use std::{process::ExitStatus, sync::Arc};

use crate::solipath_shell::{command_executor::CommandExecutor, install_command_executor::InstallCommandExecutor, install_command_filter::InstallCommandFilter, looping_install_command_executor::{LoopingInstallCommandExecutor, LoopingInstallCommandExecutorTrait}};
use crate::solipath_shell::command_executor::CommandExecutorTrait;
use crate::solipath_download::dependency_downloader::DependencyDownloader;
use crate::solipath_download::looping_dependency_downloader::LoopingDependencyDownloader;
use crate::solipath_download::looping_dependency_downloader::LoopingDependencyDownloaderTrait;
use crate::solipath_directory::solipath_directory_finder::SolipathDirectoryFinder;
use crate::solipath_directory::solipath_directory_finder::SolipathDirectoryFinderTrait;
use crate::solipath_download::conditional_file_downloader::ConditionalFileDownloader;
use crate::solipath_download::file_decompressor::FileDecompressor;
use crate::solipath_download::file_downloader::FileDownloader;
use crate::solipath_download::file_to_string_downloader::FileToStringDownloader;
use crate::solipath_environment_variable::environment_setter::EnvironmentSetter;
use crate::solipath_environment_variable::looping_environment_setter::LoopingEnvironmentSetter;
use crate::solipath_environment_variable::looping_environment_setter::LoopingEnvironmentSetterTrait;
use crate::solipath_instructions::dependency_instructions_retriever::DependencyInstructionsRetriever;
use crate::solipath_instructions::looping_dependency_instructions_retriever::LoopingDependencyInstructionsRetriever;
use crate::solipath_instructions::looping_dependency_instructions_retriever::LoopingDependencyInstructionsRetrieverTrait;
use crate::solipath_template::looping_template_retriever::LoopingTemplateRetriever;
use crate::solipath_template::looping_template_retriever::LoopingTemplateRetrieverTrait;
use crate::solipath_template::template_retriever::TemplateRetriever;
use crate::solipath_template::template_variable_replacer::TemplateVariableReplacer;
use crate::{
    solipath_dependency_metadata::dependency::Dependency,
    solipath_platform::{current_platform_retriever::CurrentPlatformRetriever, platform_filter::PlatformFilter},
};

pub struct CommandWithPathExecutor {
    dependency_instructions_list_retriever: Arc<dyn LoopingDependencyInstructionsRetrieverTrait + Sync + Send>,
    looping_template_retriever: Arc<dyn LoopingTemplateRetrieverTrait + Sync + Send>,
    looping_dependency_downloader: Arc<dyn LoopingDependencyDownloaderTrait + Sync + Send>,
    looping_environment_setter: Arc<dyn LoopingEnvironmentSetterTrait + Sync + Send>,
    looping_install_command_executor: Arc<dyn LoopingInstallCommandExecutorTrait + Sync + Send>,
    command_executor: Arc<dyn CommandExecutorTrait + Sync + Send>,
}

impl CommandWithPathExecutor {
    pub fn new() -> Self {
        let directory_finder = Arc::new(SolipathDirectoryFinder::new());
        let command_executor = Arc::new(CommandExecutor::new());
        CommandWithPathExecutor::new_with_directory_finder(directory_finder, command_executor)
    }
    pub fn new_with_directory_finder(
        directory_finder: Arc<dyn SolipathDirectoryFinderTrait + Sync + Send>,
        command_executor: Arc<dyn CommandExecutorTrait + Sync + Send>,
    ) -> Self {
        let file_downloader = Arc::new(FileDownloader::new());
        let file_decompressor = Arc::new(FileDecompressor::new());
        let conditional_file_downloader = Arc::new(ConditionalFileDownloader::new(file_downloader, file_decompressor));
        let file_to_string_downloader = Arc::new(FileToStringDownloader::new(conditional_file_downloader.clone()));
        let current_platform_retriever = Arc::new(CurrentPlatformRetriever::new());
        let platform_filter = Arc::new(PlatformFilter::new(current_platform_retriever));
        let template_variable_replacer = Arc::new(TemplateVariableReplacer::new());
        let template_retriever = Arc::new(TemplateRetriever::new(
            file_to_string_downloader.clone(),
            directory_finder.clone(),
            template_variable_replacer,
        ));
        let looping_template_retriever = Arc::new(LoopingTemplateRetriever::new(
            template_retriever,
            platform_filter.clone(),
        ));
        let dependency_instructions_retriever = Arc::new(DependencyInstructionsRetriever::new(
            file_to_string_downloader,
            directory_finder.clone(),
        ));
        let looping_dependency_instructions_retriever = Arc::new(LoopingDependencyInstructionsRetriever::new(
            dependency_instructions_retriever,
            platform_filter.clone(),
        ));
        let dependency_downloader = Arc::new(DependencyDownloader::new(
            directory_finder.clone(),
            conditional_file_downloader,
        ));
        let looping_dependency_downloader = Arc::new(LoopingDependencyDownloader::new(
            dependency_downloader,
            platform_filter.clone(),
        ));
        let environment_setter = Arc::new(EnvironmentSetter::new(directory_finder.clone()));
        let looping_environment_setter = Arc::new(LoopingEnvironmentSetter::new(environment_setter, platform_filter.clone()));
        let install_command_filter = Arc::new(InstallCommandFilter::new(directory_finder.clone()));
        let install_command_executor = Arc::new(InstallCommandExecutor::new(command_executor.clone(), install_command_filter, directory_finder));
        let looping_install_command_executor = Arc::new(LoopingInstallCommandExecutor::new(install_command_executor, platform_filter));
        Self {
            dependency_instructions_list_retriever: looping_dependency_instructions_retriever,
            looping_template_retriever,
            looping_dependency_downloader,
            looping_environment_setter,
            looping_install_command_executor,
            command_executor,
        }
    }

    pub async fn set_path_from_solipath_file_and_execute_command(&self, commands: &[String]) -> ExitStatus {
        let file_contents =
        std::fs::read_to_string("solipath.json").expect("could not find a solipath.json file in current directory");
        let dependency_list: Vec<Dependency> =
            serde_json::from_str(&file_contents).expect("failed to parse dependency file");
        self.set_path_and_execute_command(dependency_list, commands).await
    }

    pub async fn set_path_and_execute_command(&self, dependency_list: Vec<Dependency>, commands: &[String]) -> ExitStatus {
        let mut dependency_instructions_list = self
            .dependency_instructions_list_retriever
            .retrieve_dependency_instructions_list(&dependency_list)
            .await;
        dependency_instructions_list.append(
            &mut self
                .looping_template_retriever
                .retrieve_instructions_from_templates(&dependency_instructions_list)
                .await,
        );
        
        self.looping_dependency_downloader
            .download_dependencies(&dependency_instructions_list)
            .await;
        self.looping_environment_setter
            .set_environment_variables(&dependency_instructions_list);
        self.looping_install_command_executor
            .run_install_commands(&dependency_instructions_list);
        self.command_executor.execute_command(&commands)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solipath_shell::command_executor::MockCommandExecutorTrait;
    use crate::solipath_shell::looping_install_command_executor::MockLoopingInstallCommandExecutorTrait;
    use crate::solipath_download::looping_dependency_downloader::MockLoopingDependencyDownloaderTrait;
    use crate::solipath_environment_variable::looping_environment_setter::MockLoopingEnvironmentSetterTrait;
    use crate::solipath_instructions::data::dependency_instructions::DependencyInstructions;
    use crate::solipath_instructions::data::install_instructions::InstallInstructions;
    use crate::solipath_instructions::looping_dependency_instructions_retriever::MockLoopingDependencyInstructionsRetrieverTrait;
    use crate::solipath_template::looping_template_retriever::MockLoopingTemplateRetrieverTrait;
    use mockall::predicate::*;

    #[tokio::test]
    async fn get_dependency_instructions_downloads_dependencies_sets_environment_variables_then_executes_the_command() {
        let dependency = Dependency::new("java", "11");
        let dependency2 = Dependency::new("node", "12");
        let dependency_list = vec![dependency.clone()];
        let instructions = serde_json::from_str::<InstallInstructions>(
            r#"
            {"downloads": [
                {"url": "www.github.com/node15.zip", "destination_directory": "node15Folder"}
            ]}"#,
        )
        .unwrap();
        let dependency_instructions = vec![DependencyInstructions::new(dependency.clone(), instructions.clone())];
        let dependency_instructions2 = vec![DependencyInstructions::new(dependency2.clone(), instructions.clone())];
        let mut combined_instructions = Vec::new();
        combined_instructions.extend(dependency_instructions.iter().cloned());
        combined_instructions.extend(dependency_instructions2.iter().cloned());
        let commands = vec!["command1".to_string(), "argument1".to_string()];
        let mut dependency_instructions_list_retriever = MockLoopingDependencyInstructionsRetrieverTrait::new();
        dependency_instructions_list_retriever
            .expect_retrieve_dependency_instructions_list()
            .with(eq(dependency_list.clone()))
            .return_const(dependency_instructions.clone());
        let mut looping_template_retriever = MockLoopingTemplateRetrieverTrait::new();
        looping_template_retriever
            .expect_retrieve_instructions_from_templates()
            .with(eq(dependency_instructions.clone()))
            .return_const(dependency_instructions2);
        let mut looping_dependency_downloader = MockLoopingDependencyDownloaderTrait::new();
        looping_dependency_downloader
            .expect_download_dependencies()
            .with(eq(combined_instructions.clone()))
            .return_const(());
        let mut looping_environment_setter = MockLoopingEnvironmentSetterTrait::new();
        looping_environment_setter
            .expect_set_environment_variables()
            .with(eq(combined_instructions.clone()))
            .return_const(());
        let mut looping_command_executor = MockLoopingInstallCommandExecutorTrait::new();
        looping_command_executor
            .expect_run_install_commands()
            .with(eq(combined_instructions.clone()))
            .return_const(());

        let mut command_executor = MockCommandExecutorTrait::new();
        command_executor
            .expect_execute_command()
            .withf(|actual| actual == vec!["command1".to_string(), "argument1".to_string()])
            .return_const(ExitStatus::default());
        let command_with_path_executor = CommandWithPathExecutor {
            dependency_instructions_list_retriever: Arc::new(dependency_instructions_list_retriever),
            looping_template_retriever: Arc::new(looping_template_retriever),
            looping_dependency_downloader: Arc::new(looping_dependency_downloader),
            looping_environment_setter: Arc::new(looping_environment_setter),
            looping_install_command_executor: Arc::new(looping_command_executor),
            command_executor: Arc::new(command_executor),
        };
        command_with_path_executor
            .set_path_and_execute_command(dependency_list, &commands)
            .await;
    }
}
