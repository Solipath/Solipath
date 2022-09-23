use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::Mutex;
use tempfile::tempdir;

use solipath_lib::solipath_commandline::command_executor::CommandExecutorTrait;
use solipath_lib::solipath_dependency_metadata::dependency::Dependency;
use solipath_lib::solipath_directory::solipath_directory_finder::SolipathDirectoryFinderTrait;
use solipath_lib::solipath_execute::command_with_path_executor::CommandWithPathExecutor;

//tests in this file are integration tests that pull down a couple hundred megabytes of data.
//I don't want to run these every time. These can be run with "cargo test -- --features=expensive_tests"
#[tokio::test]
#[cfg_attr(not(feature = "expensive_tests"), ignore)]
async fn install_node_integration_test() {
    let directory_finder = IntegrationTestSolipathDirectoryFinder::new(tempdir().unwrap().into_path());
    let command_executor = Arc::new(IntegrationTestCommandExecutor::new());

    let command_with_path_executor =
        CommandWithPathExecutor::new_with_directory_finder(Arc::new(directory_finder), command_executor.clone());
    let arguments = vec!["node".to_string(), "--version".to_string()];
    let dependency_list = vec![Dependency::new("node", "15.11.0")];
    command_with_path_executor
        .set_path_and_execute_command(dependency_list, &arguments)
        .await;

    let output = command_executor.get_output();
    if std::env::consts::OS == "windows" {
        assert_eq!(output, "v15.11.0\r\n");
    } else {
        assert_eq!(output, "v15.11.0\n");
    }
}

#[tokio::test]
#[cfg_attr(not(feature = "expensive_tests"), ignore)]
async fn install_java_integration_test() {
    let directory_finder = IntegrationTestSolipathDirectoryFinder::new(tempdir().unwrap().into_path());
    let command_executor = Arc::new(IntegrationTestCommandExecutor::new());

    let command_with_path_executor =
        CommandWithPathExecutor::new_with_directory_finder(Arc::new(directory_finder), command_executor.clone());
    let arguments = vec!["java".to_string(), "--version".to_string()];
    let dependency_list = vec![Dependency::new("java", "17")];
    command_with_path_executor
        .set_path_and_execute_command(dependency_list, &arguments)
        .await;

    let output = command_executor.get_output();
    assert!(output.starts_with("openjdk 17.0.4.1 2022-08-12"));
}

struct IntegrationTestSolipathDirectoryFinder {
    base_dir: PathBuf,
}

impl IntegrationTestSolipathDirectoryFinder {
    fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }
}

impl SolipathDirectoryFinderTrait for IntegrationTestSolipathDirectoryFinder {
    fn get_base_solipath_directory(&self) -> PathBuf {
        self.base_dir.clone()
    }
}

struct IntegrationTestCommandExecutor {
    output: Arc<Mutex<String>>,
}

impl IntegrationTestCommandExecutor {
    fn new() -> Self {
        Self {
            output: Arc::new(Mutex::new("".to_string())),
        }
    }

    fn get_output(&self) -> String {
        self.output.lock().unwrap().to_string()
    }
}

impl CommandExecutorTrait for IntegrationTestCommandExecutor {
    fn execute_command(&self, commands: &[String]) {
        let mut command = if std::env::consts::OS == "windows" {
            let mut command = Command::new("cmd");
            command.arg("/C").args(commands);
            command
        } else {
            let mut command = Command::new(commands.get(0).expect("expected at least one command!"));
            command.args(&commands[1..]);
            command
        };
        command.stdout(Stdio::piped());
        command.stdin(Stdio::piped());

        let output = command.output().expect("could not retrieve command output");
        *self.output.lock().unwrap() = String::from_utf8_lossy(&output.stdout).to_string()
    }

    fn execute_single_string_command(&self, _: String) {
    
    }
}
