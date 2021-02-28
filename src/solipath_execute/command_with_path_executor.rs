use std::sync::Arc;

use crate::solipath_commandline::command_executor::CommandExecutor;
use crate::solipath_commandline::command_executor::CommandExecutorTrait;
use crate::solipath_dependency_download::dependency_downloader::DependencyDownloader;
use crate::solipath_dependency_download::looping_dependency_downloader::LoopingDependencyDownloader;
use crate::solipath_dependency_download::looping_dependency_downloader::LoopingDependencyDownloaderTrait;
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
use crate::{
    solipath_dependency_metadata::dependency::Dependency,
    solipath_platform::{current_platform_retriever::CurrentPlatformRetriever, platform_filter::PlatformFilter},
};

pub struct CommandWithPathExecutor {
    dependency_instructions_list_retriever: Arc<dyn LoopingDependencyInstructionsRetrieverTrait + Sync + Send>,
    looping_dependency_downloader: Arc<dyn LoopingDependencyDownloaderTrait + Sync + Send>,
    looping_environment_setter: Arc<dyn LoopingEnvironmentSetterTrait + Sync + Send>,
    command_executor: Arc<dyn CommandExecutorTrait + Sync + Send>,
}

impl CommandWithPathExecutor {
    pub fn new() -> Self {
        let directory_finder = Arc::new(SolipathDirectoryFinder::new());
        CommandWithPathExecutor::new_with_directory_finder(directory_finder)
    }
    pub fn new_with_directory_finder(directory_finder: Arc<dyn SolipathDirectoryFinderTrait + Sync + Send>) -> Self {
        let file_downloader = Arc::new(FileDownloader::new());
        let file_decompressor = Arc::new(FileDecompressor::new());
        let conditional_file_downloader = Arc::new(ConditionalFileDownloader::new(file_downloader, file_decompressor));
        let file_to_string_downloader = Arc::new(FileToStringDownloader::new(conditional_file_downloader.clone()));
        let current_platform_retriever = Arc::new(CurrentPlatformRetriever::new());
        let platform_filter = Arc::new(PlatformFilter::new(current_platform_retriever));
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
        let environment_setter = Arc::new(EnvironmentSetter::new(directory_finder));
        let looping_environment_setter = Arc::new(LoopingEnvironmentSetter::new(environment_setter, platform_filter));
        let command_executor = Arc::new(CommandExecutor::new());

        Self {
            dependency_instructions_list_retriever: looping_dependency_instructions_retriever,
            looping_dependency_downloader,
            looping_environment_setter,
            command_executor,
        }
    }

    pub async fn set_path_and_execute_command(&self, dependency_list: Vec<Dependency>, commands: &[String]) {
        let dependency_instructions_list = self
            .dependency_instructions_list_retriever
            .retrieve_dependency_instructions_list(dependency_list)
            .await;
        self.looping_dependency_downloader
            .download_dependencies(dependency_instructions_list.clone())
            .await;
        self.looping_environment_setter
            .set_environment_variables(dependency_instructions_list);
        self.command_executor.execute_command(commands);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solipath_commandline::command_executor::MockCommandExecutorTrait;
    use crate::solipath_dependency_download::looping_dependency_downloader::MockLoopingDependencyDownloaderTrait;
    use crate::solipath_environment_variable::looping_environment_setter::MockLoopingEnvironmentSetterTrait;
    use crate::solipath_instructions::data::dependency_instructions::DependencyInstructions;
    use crate::solipath_instructions::data::install_instructions::InstallInstructions;
    use crate::solipath_instructions::looping_dependency_instructions_retriever::MockLoopingDependencyInstructionsRetrieverTrait;
    use mockall::predicate::*;

    #[tokio::test]
    async fn get_dependency_instructions_downloads_dependencies_sets_environment_variables_then_executes_the_command() {
        let dependency = Dependency::new("java", "11");
        let dependency_list = vec![dependency.clone()];
        let instructions = serde_json::from_str::<InstallInstructions>(
            r#"
            {"downloads": [
                {"url": "www.github.com/node15.zip", "destination_directory": "node15Folder"}
            ]}"#,
        )
        .unwrap();
        let dependency_instructions = vec![DependencyInstructions::new(dependency.clone(), instructions.clone())];
        let commands = vec!["command1".to_string(), "argument1".to_string()];
        let mut dependency_instructions_list_retriever = MockLoopingDependencyInstructionsRetrieverTrait::new();
        dependency_instructions_list_retriever
            .expect_retrieve_dependency_instructions_list()
            .with(eq(dependency_list.clone()))
            .return_const(dependency_instructions.clone());
        let mut looping_dependency_downloader = MockLoopingDependencyDownloaderTrait::new();
        looping_dependency_downloader
            .expect_download_dependencies()
            .with(eq(dependency_instructions.clone()))
            .return_const(());
        let mut looping_environment_setter = MockLoopingEnvironmentSetterTrait::new();
        looping_environment_setter
            .expect_set_environment_variables()
            .with(eq(dependency_instructions.clone()))
            .return_const(());
        let mut command_executor = MockCommandExecutorTrait::new();
        command_executor
            .expect_execute_command()
            .withf(|actual| actual == vec!["command1".to_string(), "argument1".to_string()])
            .return_const(());
        let command_with_path_executor = CommandWithPathExecutor {
            dependency_instructions_list_retriever: Arc::new(dependency_instructions_list_retriever),
            looping_dependency_downloader: Arc::new(looping_dependency_downloader),
            looping_environment_setter: Arc::new(looping_environment_setter),
            command_executor: Arc::new(command_executor),
        };
        command_with_path_executor
            .set_path_and_execute_command(dependency_list, &commands)
            .await;
    }
}
