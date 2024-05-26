use solipath_lib::{
    solipath_cli::cli::{is_solipath_command, SolipathCli},
    solipath_execute::command_with_path_executor::CommandWithPathExecutor,
};

#[tokio::main]
async fn main() {
    let arguments: Vec<String> = std::env::args().collect();
    let arguments_without_the_solipath_executable = &arguments[1..];
    if is_solipath_command(arguments_without_the_solipath_executable) {
        SolipathCli::new()
            .run_solipath_command(&arguments_without_the_solipath_executable)
            .await
            .expect("failed to run cli command");
    } else {
        let command_with_path_executor = CommandWithPathExecutor::new();
        let exit_status = command_with_path_executor
            .set_path_from_solipath_file_and_execute_command(arguments_without_the_solipath_executable)
            .await;
        std::process::exit(exit_status.code().unwrap_or_else(|| 1));
    }
}
