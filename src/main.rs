use solipath_dependency_metadata::dependency::Dependency;
use solipath_execute::command_with_path_executor::CommandWithPathExecutor;

mod solipath_commandline;
mod solipath_dependency_download;
mod solipath_dependency_metadata;
mod solipath_directory;
mod solipath_download;
mod solipath_environment_variable;
mod solipath_execute;
mod solipath_instructions;
mod solipath_platform;

#[tokio::main]
async fn main() {
    let command_with_path_executor = CommandWithPathExecutor::new();
    let file_contents =
        std::fs::read_to_string("solipath.json").expect("could not find a solipath.json file in current directory");
    let dependency_list: Vec<Dependency> =
        serde_json::from_str(&file_contents).expect("failed to parse dependency file");
    let arguments: Vec<String> = std::env::args().collect();
    command_with_path_executor
        .set_path_and_execute_command(dependency_list, &arguments[1..])
        .await;
}
