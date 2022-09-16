#[cfg(test)]
use mockall::automock;
use std::process::Command;
use std::process::Stdio;

#[cfg_attr(test, automock)]
pub trait CommandExecutorTrait {
    fn execute_command(&self, commands: &[String]);
    fn execute_single_string_command(&self, command: String);
}

pub struct CommandExecutor;

impl CommandExecutor {
    pub fn new() -> Self {
        Self {}
    }

    pub fn setup_command(&self, commands: &[String]) -> Command {
        if std::env::consts::OS == "windows" {
            let mut command = Command::new("cmd");
            command.arg("/C").args(commands);
            command
        } else {
            let mut command = Command::new(commands.get(0).expect("expected at least one command!"));
            command.args(&commands[1..]);
            command
        }
    }

    pub fn setup_single_string_command(&self, commands: &String) -> Command {
        if std::env::consts::OS == "windows" {
            let mut command = Command::new("cmd");
            command.arg("/C").arg(commands);
            command
        } else {
            let mut command = Command::new("bash");
            command.arg("-c").arg(commands);
            command
        }
    }

    pub fn run_command(&self, command: &mut Command){
        command
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .status()
            .unwrap_or_else(|error| panic!("failed to execute the command: {:?}, error: {}", command, error));
    }
}

#[cfg_attr(test, automock)]
impl CommandExecutorTrait for CommandExecutor {
    fn execute_command(&self, commands: &[String]) {
        self.run_command(&mut self.setup_command(commands));
    }

    fn execute_single_string_command(&self, command: String) {
        self.run_command(&mut self.setup_single_string_command(&command));
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn run_command() {
        let command_executor = CommandExecutor::new();
        let mut command = command_executor.setup_command(&vec!["echo".to_string(), "the test worked!!!".to_string()]);
        let output = command.stdout(Stdio::piped()).output().expect("failed to run command");
        if std::env::consts::OS == "windows" {
            assert_eq!(String::from_utf8_lossy(&output.stdout), "\"the test worked!!!\"\r\n");
        } else {
            assert_eq!(String::from_utf8_lossy(&output.stdout), "the test worked!!!\n");
        }
        
    }

    #[test]
    fn run_single_string_command() {
        let command_executor = CommandExecutor::new();
        if std::env::consts::OS == "windows" {
            let mut command = command_executor.setup_single_string_command(&"cd tests && dir /b".to_string());
            let output = command.stdout(Stdio::piped()).output().expect("failed to run command");
            assert_eq!(String::from_utf8_lossy(&output.stdout), "mod.rs\r\nresources\r\n");
        } else {
            let mut command = command_executor.setup_single_string_command(&"cd tests && ls".to_string());
            let output = command.stdout(Stdio::piped()).output().expect("failed to run command");
            assert_eq!(String::from_utf8_lossy(&output.stdout), "mod.rs\nresources\n");
        }
    }
}
