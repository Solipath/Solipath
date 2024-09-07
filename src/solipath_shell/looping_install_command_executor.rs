use std::sync::Arc;

#[cfg(test)]
use mockall::automock;

use crate::{solipath_shell::install_command_executor::InstallCommandExecutorTrait, solipath_instructions::data::dependency_instructions::DependencyInstructions};
use crate::solipath_platform::platform_filter::PlatformFilterTrait;


#[cfg_attr(test, automock)]
pub trait LoopingInstallCommandExecutorTrait {
    fn run_install_commands(&self, dependency_instructions_list: &Vec<DependencyInstructions>);
}

pub struct LoopingInstallCommandExecutor {
    install_command_executor: Arc<dyn InstallCommandExecutorTrait + Send + Sync>,
    platform_filter: Arc<dyn PlatformFilterTrait + Send + Sync>,
}

impl LoopingInstallCommandExecutor {
    pub fn new(
        install_command_executor: Arc<dyn InstallCommandExecutorTrait + Send + Sync>,
        platform_filter: Arc<dyn PlatformFilterTrait + Send + Sync>) -> Self{
            Self{install_command_executor, platform_filter}
    }

    fn run_single_install_command(&self, dependency_instructions: &DependencyInstructions){
        let dependency = dependency_instructions.get_dependency();
        dependency_instructions.get_install_commands()
        .iter()
        .filter(|install_command|self.platform_filter.current_platform_is_match(install_command.get_platform_filters()))
        .for_each(move |install_command|
            self.install_command_executor.execute_command(&dependency, &install_command)
        );
    }
}

impl LoopingInstallCommandExecutorTrait for LoopingInstallCommandExecutor {
    fn run_install_commands(&self, dependency_instructions_list: &Vec<DependencyInstructions>){
        dependency_instructions_list
            .iter()
            .for_each(|dependency_instructions| {
                self.run_single_install_command(&dependency_instructions);
            });
    }
}


#[cfg(test)]
mod tests {
    use mockall::predicate::eq;

    use super::*;
    use crate::{solipath_dependency_metadata::dependency::Dependency, solipath_instructions::data::install_command::InstallCommand, solipath_platform::platform::Platform};
    use crate::solipath_instructions::data::install_instructions::InstallInstructions;
    use crate::solipath_platform::platform_filter::MockPlatformFilterTrait;
    use crate::solipath_shell::install_command_executor::MockInstallCommandExecutorTrait;

    use crate::solipath_platform::platform_filter::mock::verify_platform_filter;


    #[test]
    fn can_set_command_executor() {
        let dependency = Dependency::new("dependency", "123.12");
        let json = r#"{"install_commands": [{"command": "do something"}]}"#;
        let dependency_instructions = DependencyInstructions::new(
            dependency.clone(),
            serde_json::from_str::<InstallInstructions>(json).unwrap(),
        );
        let dependency_instructions_list: Vec<DependencyInstructions> = vec![dependency_instructions.clone()];
        let mut install_command_executor = MockInstallCommandExecutorTrait::new();
        verify_install_command_called(
            &mut install_command_executor,
            &dependency,
            r#"{"command": "do something"}"#,
        );

        let mut platform_filter = MockPlatformFilterTrait::new();
        verify_platform_filter(&mut platform_filter, Vec::new(), true, 1);
        let looping_install_command_executor =
            LoopingInstallCommandExecutor::new(Arc::new(install_command_executor), Arc::new(platform_filter));
        looping_install_command_executor.run_install_commands(&dependency_instructions_list);
    }


    #[test]
    fn can_set_command_executor_loops() {
        let dependency = Dependency::new("dependency", "123.12");
        let json = r#"{"install_commands": [
            {"command": "do something"},
            {"command": "do something2"}
        ]}"#;
        let dependency_instructions = DependencyInstructions::new(
            dependency.clone(),
            serde_json::from_str::<InstallInstructions>(json).unwrap(),
        );
        let dependency_instructions_list: Vec<DependencyInstructions> = vec![dependency_instructions.clone()];
        let mut install_command_executor = MockInstallCommandExecutorTrait::new();
        verify_install_command_called(
            &mut install_command_executor,
            &dependency,
            r#"{"command": "do something"}"#,
        );
        verify_install_command_called(
            &mut install_command_executor,
            &dependency,
            r#"{"command": "do something2"}"#,
        );

        let mut platform_filter = MockPlatformFilterTrait::new();
        verify_platform_filter(&mut platform_filter, Vec::new(), true, 2);
        let looping_install_command_executor =
            LoopingInstallCommandExecutor::new(Arc::new(install_command_executor), Arc::new(platform_filter));
        looping_install_command_executor.run_install_commands(&dependency_instructions_list);
    }

    #[test]
    fn can_set_command_executor_and_filter_result() {
        let dependency = Dependency::new("dependency", "123.12");
        let json = r#"{"install_commands": [
            {"command": "do something", "platform_filters": [{"os": "a good match", "arch": "x86"}]},
            {"command": "do something2", "platform_filters": [{"os": "a bad match", "arch": "x86"}]}
        ]}"#;
        let dependency_instructions = DependencyInstructions::new(
            dependency.clone(),
            serde_json::from_str::<InstallInstructions>(json).unwrap(),
        );
        let dependency_instructions_list: Vec<DependencyInstructions> = vec![dependency_instructions.clone()];
        let mut install_command_executor = MockInstallCommandExecutorTrait::new();
        verify_install_command_called(
            &mut install_command_executor,
            &dependency,
            r#"{"command": "do something", "platform_filters": [{"os": "a good match", "arch": "x86"}]}"#,
        );

        let mut platform_filter = MockPlatformFilterTrait::new();
        verify_platform_filter(
            &mut platform_filter,
            vec![Platform::new("a good match", "x86")],
            true,
            1,
        );
        verify_platform_filter(
            &mut platform_filter,
            vec![Platform::new("a bad match", "x86")],
            false,
            1,
        );
        let looping_install_command_executor =
            LoopingInstallCommandExecutor::new(Arc::new(install_command_executor), Arc::new(platform_filter));
        looping_install_command_executor.run_install_commands(&dependency_instructions_list);
    }


    fn verify_install_command_called(
        command_executor: &mut MockInstallCommandExecutorTrait,
        dependency: &Dependency,
        json: &str,
    ) {
        command_executor
            .expect_execute_command()
            .with(
                eq(dependency.clone()),
                eq(serde_json::from_str::<InstallCommand>(json).unwrap()),
            )
            .times(1)
            .return_const(());
    }
}