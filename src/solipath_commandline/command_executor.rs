#[cfg(test)]
use mockall::automock;
use std::process::Command;
use std::process::Stdio;

#[cfg_attr(test, automock)]
pub trait CommandExecutorTrait {
    fn execute_command(&self, commands: &[String]);
}

pub struct CommandExecutor;

impl CommandExecutor {
    pub fn new() -> Self {
        Self {}
    }

    fn setup_command(&self, commands: &[String]) -> Command {
        let mut command = Command::new(commands.get(0).expect("expected at least one command!"));
        command
            .args(&commands[1..])
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit());
        command
    }
}

#[cfg_attr(test, automock)]
impl CommandExecutorTrait for CommandExecutor {
    fn execute_command(&self, commands: &[String]) {
        self.setup_command(commands)
            .status()
            .expect("failed to execute the command!");
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
        assert_eq!(String::from_utf8_lossy(&output.stdout), "the test worked!!!\n");
    }
}
