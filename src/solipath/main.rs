use solipath_lib::solipath_execute::command_with_path_executor::CommandWithPathExecutor;

#[tokio::main]
async fn main() {
    let arguments: Vec<String> = std::env::args().collect();
    
    let command_with_path_executor = CommandWithPathExecutor::new();
    let exit_status = command_with_path_executor
        .set_path_from_solipath_file_and_execute_command(&arguments[1..])
        .await;
    std::process::exit(exit_status.code().unwrap_or_else(||1));
}
