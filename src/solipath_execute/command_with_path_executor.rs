use std::{process::ExitStatus, sync::Arc};

use crate::solipath_instructions::data::dependency::Dependency;
use crate::{
    async_loop::run_async,
    solipath_directory::solipath_directory_finder::{SolipathDirectoryFinder, SolipathDirectoryFinderTrait},
    solipath_download::{
        conditional_file_downloader::ConditionalFileDownloader,
        dependency_downloader::{DependencyDownloader, DependencyDownloaderTrait},
        file_decompressor::FileDecompressor,
        file_downloader::FileDownloader,
        file_to_string_downloader::FileToStringDownloader,
    },
    solipath_environment_variable::environment_setter::{EnvironmentSetter, EnvironmentSetterTrait},
    solipath_instructions::{
        data::dependency_instructions::{DependencyInstructions, VecDependencyInstructions},
        dependency_instructions_retriever::{DependencyInstructionsRetriever, DependencyInstructionsRetrieverTrait},
    },
    solipath_platform::{
        current_platform_retriever::{CurrentPlatformRetriever, CurrentPlatformRetrieverTrait},
        platform_filter::{filter_list, PlatformFilter, PlatformFilterTrait},
    },
    solipath_shell::{
        command_executor::{CommandExecutor, CommandExecutorTrait},
        install_command_executor::{InstallCommandExecutor, InstallCommandExecutorTrait},
        install_command_filter::InstallCommandFilter,
    },
    solipath_template::{
        template_retriever::{TemplateRetriever, TemplateRetrieverTrait},
        template_variable_replacer::TemplateVariableReplacer,
    },
};

pub struct CommandWithPathExecutor {
    platform_filter: Arc<dyn PlatformFilterTrait>,
    dependency_instructions_retriever: Arc<dyn DependencyInstructionsRetrieverTrait>,
    template_retriever: Arc<dyn TemplateRetrieverTrait>,
    dependency_downloader: Arc<dyn DependencyDownloaderTrait>,
    environment_setter: Arc<dyn EnvironmentSetterTrait>,
    install_command_executor: Arc<dyn InstallCommandExecutorTrait>,
    command_executor: Arc<dyn CommandExecutorTrait>,
}

impl CommandWithPathExecutor {
    pub fn new() -> Self {
        let base_solipath_url =
            "https://raw.githubusercontent.com/Solipath/Solipath-Install-Instructions/main".to_string();
        let directory_finder = Arc::new(SolipathDirectoryFinder::new());
        let platform_retriever = Arc::new(CurrentPlatformRetriever::new());
        let command_executor = Arc::new(CommandExecutor::new());
        Self::new_with_injected_values(
            base_solipath_url,
            directory_finder,
            platform_retriever,
            command_executor,
        )
    }

    pub async fn set_path_from_solipath_file_and_execute_command(&self, commands: &[String]) -> ExitStatus {
        let file_contents =
            std::fs::read_to_string("solipath.json").expect("could not find a solipath.json file in current directory");
        let dependency_list: Vec<Dependency> =
            serde_json::from_str(&file_contents).expect("failed to parse dependency file");
        self.set_path_and_execute_command(dependency_list, commands).await
    }

    pub fn new_with_injected_values(
        base_solipath_url: String,
        directory_finder: Arc<dyn SolipathDirectoryFinderTrait + Send + Sync>,
        platform_retriever: Arc<dyn CurrentPlatformRetrieverTrait + Send + Sync>,
        command_executor: Arc<dyn CommandExecutorTrait + Send + Sync>,
    ) -> Self {
        let file_downloader = Arc::new(FileDownloader::new());
        let file_decompressor = Arc::new(FileDecompressor::new());
        let conditional_file_downloader = Arc::new(ConditionalFileDownloader::new(file_downloader, file_decompressor));
        let file_to_string_downloader = Arc::new(FileToStringDownloader::new(conditional_file_downloader.clone()));
        let dependency_instructions_retriever = Arc::new(DependencyInstructionsRetriever::new_with_alternate_url(
            base_solipath_url.clone(),
            file_to_string_downloader.clone(),
            directory_finder.clone(),
        ));
        let template_retriever = Arc::new(TemplateRetriever::new_with_alternate_url(
            base_solipath_url,
            file_to_string_downloader,
            directory_finder.clone(),
            Arc::new(TemplateVariableReplacer::new()),
        ));
        let platform_filter = Arc::new(PlatformFilter::new(platform_retriever));
        let dependency_downloader = Arc::new(DependencyDownloader::new(
            directory_finder.clone(),
            conditional_file_downloader,
        ));

        let environment_setter = Arc::new(EnvironmentSetter::new(directory_finder.clone()));

        let install_command_filter = Arc::new(InstallCommandFilter::new(directory_finder.clone()));
        let install_command_executor = Arc::new(InstallCommandExecutor::new(
            command_executor.clone(),
            install_command_filter,
            directory_finder,
        ));

        CommandWithPathExecutor {
            platform_filter,
            dependency_instructions_retriever,
            template_retriever,
            dependency_downloader,
            environment_setter,
            install_command_executor,
            command_executor,
        }
    }

    async fn get_dependency_instructions(&self, dependency_list: &Vec<Dependency>) -> Vec<DependencyInstructions> {
        let dependency_list = filter_list(&self.platform_filter, &dependency_list);
        let mut dependency_instructions = run_async(&dependency_list, |dependency| {
            self.dependency_instructions_retriever
                .retrieve_dependency_instructions(dependency)
        })
        .await
        .filter_platform(&self.platform_filter);
        let mut template_instructions =
            run_async(&dependency_instructions.get_templates(), |(dependency, template)| {
                self.template_retriever
                    .retrieve_instructions_from_template(dependency, template)
            })
            .await
            .filter_platform(&self.platform_filter);
        dependency_instructions.append(&mut template_instructions);
        dependency_instructions
    }

    pub async fn set_path_and_execute_command(
        &self,
        dependency_list: Vec<Dependency>,
        commands: &[String],
    ) -> ExitStatus {
        let dependency_instructions = self.get_dependency_instructions(&dependency_list).await;

        run_async(
            &dependency_instructions.get_downloads(),
            |(dependency, download_instruction)| {
                self.dependency_downloader
                    .download_dependency(dependency, download_instruction)
            },
        )
        .await;

        dependency_instructions
            .get_environment_variables()
            .iter()
            .for_each(|(dependency, environment_variable)| {
                self.environment_setter.set_variable(dependency, environment_variable)
            });
        dependency_instructions
            .get_install_commands()
            .iter()
            .for_each(|(dependency, install_command)| {
                self.install_command_executor
                    .execute_command(dependency, install_command);
            });
        self.command_executor.execute_command(commands)
    }
}

#[cfg(test)]
mod test {
    use std::{
        env::{self, VarError},
        fs::{read_dir, read_to_string},
        path::PathBuf,
        process::ExitStatus, thread::sleep, time::Duration,
    };

    use tempfile::tempdir;
    use tokio::task::JoinHandle;
    use warp::Filter;

    use crate::{
        path_buf_ext::PathBufExt,
        solipath_directory::moveable_home_directory_finder::MoveableHomeDirectoryFinder,
        solipath_platform::{platform::Platform, platform_filter::mock::FakeCurrentPlatformRetriever},
        solipath_shell::command_executor::pub_test::MockCommandExecutor,
    };

    use super::*;

    impl CommandWithPathExecutor {
        fn new_test(
            output_path: &PathBuf,
            base_solipath_url: String,
            command_executor: Arc<dyn CommandExecutorTrait + Send + Sync>,
        ) -> Self {
            let directory_finder = Arc::new(MoveableHomeDirectoryFinder::new(output_path.clone()));
            let platform_retriever = Arc::new(FakeCurrentPlatformRetriever {
                platform: Platform::new("Matching OS", "Matching Arch"),
            });
            CommandWithPathExecutor::new_with_injected_values(
                base_solipath_url,
                directory_finder,
                platform_retriever,
                command_executor,
            )
        }
    }

    #[tokio::test]
    async fn test_broad_functionality_using_local_file_hosting() {
        let mock_command_executor = Arc::new(MockCommandExecutor::new());

        let output_tempdir = tempdir().unwrap();
        let output_path = output_tempdir.path().to_path_buf();
        let solipath_source =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).clone_push("tests/resources/test_solipath_with_local_downloads");
        let downloads_directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).clone_push("tests/resources");
        let base_solipath_url = "http://127.0.0.1:53123/solipath".to_string();
        let command_with_path_executor =
            CommandWithPathExecutor::new_test(&output_path, base_solipath_url, mock_command_executor.clone());
        let dependencies = serde_json::from_str::<Vec<Dependency>>(r#"
        [
            {"name": "PerfectMatchDependency", "version": "1.0.1", "platform_filters": [{"os": "Matching OS", "arch": "Matching Arch"}]},
            {"name": "BadMatchDependency", "version": "2.0.1", "platform_filters": [{"os": "a bad match", "arch": "x86"}]},
            {"name": "BadArchDependency", "version": "3.0.1", "platform_filters": [{"os": "Matching OS", "arch": "Bad Arch"}]}
        ]"#).unwrap();

        let file_server = start_file_server(&solipath_source, &downloads_directory);
        let exit_status = command_with_path_executor
            .set_path_and_execute_command(dependencies, &["command to run".to_string()])
            .await;
        file_server.abort();

        let expected_download_folder = output_path.clone_push("PerfectMatchDependency/downloads/result");
        assert_eq!(1, read_dir(expected_download_folder).unwrap().count());
        let expected_download = output_path.clone_push("PerfectMatchDependency/downloads/result/tar_bz2_file.txt");
        assert_eq!("tar bz2 file".to_string(), read_to_string(expected_download).unwrap());
        let expected_path_value = output_path.clone_push("PerfectMatchDependency/downloads/perfect_match_path");
        assert_environment_contains("PATH", &expected_path_value);

        let expected_perfect_match_path_value =
            output_path.clone_push("PerfectMatchDependency/downloads/perfect_match");
        assert_environment_contains("PERFECT_MATCH", &expected_perfect_match_path_value);
        assert_eq!(Err(VarError::NotPresent), env::var("SHOULD_NOT_BE_SET"));
        assert_eq!(
            vec![
                prefix_change_directory_command(
                    &output_path.clone_push("PerfectMatchDependency/downloads"),
                    "echo 'perfect path set!!!'"
                ),
                "command to run".to_string()
            ],
            mock_command_executor.get_commands()
        );
        assert_eq!(ExitStatus::default(), exit_status);
    }

    fn prefix_change_directory_command(directory: &PathBuf, command: &str) -> String {
        let change_directory_command = if std::env::consts::OS == "windows" {
            let expected_path_string = directory.to_str().unwrap().replace("/", "\\");
            format!("cd /d {}", expected_path_string)
        } else {
            format!("cd \"{}\"", directory.display())
        };
        format!("{} && {}", change_directory_command, command)
    }

    fn assert_environment_contains(environment_name: &str, path_value: &PathBuf) {
        let environment_variable = env::var(environment_name).unwrap();
        let mut expected_path_string = path_value.to_str().unwrap().to_string();
        if std::env::consts::OS == "windows" {
            expected_path_string = expected_path_string.replace("/", "\\");
        }
        assert!(
            environment_variable.contains(&expected_path_string),
            "{}, does not contain {:?}",
            environment_variable,
            path_value
        );
    }

    fn start_file_server(fake_solipath: &PathBuf, fake_external_server: &PathBuf) -> JoinHandle<()> {
        let route = warp::path("solipath")
            .and(warp::fs::dir(fake_solipath.clone()))
            .or(warp::path("external").and(warp::fs::dir(fake_external_server.clone())));

        let join_handle = tokio::spawn(async { warp::serve(route).run(([127, 0, 0, 1], 53123)).await });
        sleep(Duration::from_millis(1000));
        join_handle
    }
}
