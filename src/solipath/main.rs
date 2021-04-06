use solipath_lib::solipath_dependency_metadata::dependency::Dependency;
use solipath_lib::solipath_execute::command_with_path_executor::CommandWithPathExecutor;

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
