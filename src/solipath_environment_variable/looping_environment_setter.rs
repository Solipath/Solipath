use std::sync::Arc;

#[cfg(test)]
use mockall::automock;

use crate::solipath_environment_variable::environment_setter::EnvironmentSetterTrait;
use crate::solipath_instructions::data::dependency_instructions::{DependencyInstructions, VecDependencyInstructions};
use crate::solipath_platform::platform_filter::{
    run_functions_matching_platform, PlatformFilterTrait,
};

#[cfg_attr(test, automock)]
pub trait LoopingEnvironmentSetterTrait {
    fn set_environment_variables(&self, dependency_instructions_list: &Vec<DependencyInstructions>);
}

pub struct LoopingEnvironmentSetter {
    environment_setter: Arc<dyn EnvironmentSetterTrait + Send + Sync>,
    platform_filter: Arc<dyn PlatformFilterTrait + Send + Sync>,
}

impl LoopingEnvironmentSetter {
    pub fn new(
        environment_setter: Arc<dyn EnvironmentSetterTrait + Send + Sync>,
        platform_filter: Arc<dyn PlatformFilterTrait + Send + Sync>,
    ) -> Self {
        Self {
            environment_setter,
            platform_filter,
        }
    }
}

impl LoopingEnvironmentSetterTrait for LoopingEnvironmentSetter {
    fn set_environment_variables(&self, dependency_instructions_list: &Vec<DependencyInstructions>) {
        run_functions_matching_platform(
            &self.platform_filter,
            &dependency_instructions_list.get_environment_variables(),
            |(dependency, environment_variable)| 
            self.environment_setter.set_variable(dependency, environment_variable),
        );
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::solipath_dependency_metadata::dependency::Dependency;
    use crate::solipath_environment_variable::environment_setter::MockEnvironmentSetterTrait;
    use crate::solipath_instructions::data::environment_variable::EnvironmentVariable;
    use crate::solipath_instructions::data::install_instructions::InstallInstructions;
    use crate::solipath_platform::platform::Platform;
    use crate::solipath_platform::platform_filter::mock::verify_platform_filter;
    use crate::solipath_platform::platform_filter::MockPlatformFilterTrait;
    use mockall::predicate::*;

    #[test]
    fn set_single_environment_variable() {
        let dependency = Dependency::new("dependency", "123.12");
        let json = r#"
        {"environment_variables": [
            {"name": "RUST_TEST", "relative_path": "some/path/location"}
        ]}"#;
        let dependency_instructions = DependencyInstructions::new(
            dependency.clone(),
            serde_json::from_str::<InstallInstructions>(json).unwrap(),
        );
        let dependency_instructions_list: Vec<DependencyInstructions> = vec![dependency_instructions.clone()];
        let mut environment_setter = MockEnvironmentSetterTrait::new();
        verify_environment_setter_called(
            &mut environment_setter,
            &dependency,
            r#"{"name": "RUST_TEST", "relative_path": "some/path/location"}"#,
        );

        let mut platform_filter = MockPlatformFilterTrait::new();
        verify_platform_filter(&mut platform_filter, Vec::new(), true, 1);
        let looping_environment_setter =
            LoopingEnvironmentSetter::new(Arc::new(environment_setter), Arc::new(platform_filter));
        looping_environment_setter.set_environment_variables(&dependency_instructions_list);
    }

    #[test]
    fn set_single_environment_variable_one_filtered_out() {
        let dependency = Dependency::new("dependency", "123.12");
        let json = r#"
        {"environment_variables": [
            {"name": "RUST_TEST", "relative_path": "some/path/location", "platform_filters": [{"os": "a good match", "arch": "x86"}]},
            {"name": "RUST_TEST", "relative_path": "some/path/location", "platform_filters": [{"os": "a bad match", "arch": "x86"}]}
        ]}"#;
        let dependency_instructions = DependencyInstructions::new(
            dependency.clone(),
            serde_json::from_str::<InstallInstructions>(json).unwrap(),
        );
        let dependency_instructions_list: Vec<DependencyInstructions> = vec![dependency_instructions.clone()];

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

        let mut environment_setter = MockEnvironmentSetterTrait::new();

        verify_environment_setter_called(
            &mut environment_setter,
            &dependency,
            r#"{"name": "RUST_TEST", "relative_path": "some/path/location", "platform_filters": [{"os": "a good match", "arch": "x86"}]}"#,
        );

        let looping_environment_setter =
            LoopingEnvironmentSetter::new(Arc::new(environment_setter), Arc::new(platform_filter));
        looping_environment_setter.set_environment_variables(&dependency_instructions_list);
    }

    #[test]
    fn set_multiple_environment_variables() {
        let dependency1 = Dependency::new("dependency", "123.12");
        let dependency2 = Dependency::new("dependency2", "123.123");
        let json1 = r#"
        {"environment_variables": [
            {"name": "RUST_TEST", "relative_path": "some/path/location"},
            {"name": "RUST_TEST2", "relative_path": "some/path/location2"}
        ]}"#;
        let dependency_instructions1 = DependencyInstructions::new(
            dependency1.clone(),
            serde_json::from_str::<InstallInstructions>(json1).unwrap(),
        );
        let json2 = r#"
        {"environment_variables": [
            {"name": "RUST_TEST3", "relative_path": "some/path/location3"}
        ]}"#;
        let dependency_instructions2 = DependencyInstructions::new(
            dependency2.clone(),
            serde_json::from_str::<InstallInstructions>(json2).unwrap(),
        );
        let dependency_instructions_list: Vec<DependencyInstructions> =
            vec![dependency_instructions1.clone(), dependency_instructions2.clone()];
        let mut environment_setter = MockEnvironmentSetterTrait::new();

        verify_environment_setter_called(
            &mut environment_setter,
            &dependency1,
            r#"{"name": "RUST_TEST", "relative_path": "some/path/location"}"#,
        );
        verify_environment_setter_called(
            &mut environment_setter,
            &dependency1,
            r#"{"name": "RUST_TEST2", "relative_path": "some/path/location2"}"#,
        );
        verify_environment_setter_called(
            &mut environment_setter,
            &dependency2,
            r#"{"name": "RUST_TEST3", "relative_path": "some/path/location3"}"#,
        );

        let mut platform_filter = MockPlatformFilterTrait::new();
        verify_platform_filter(&mut platform_filter, Vec::new(), true, 3);
        let looping_environment_setter =
            LoopingEnvironmentSetter::new(Arc::new(environment_setter), Arc::new(platform_filter));
        looping_environment_setter.set_environment_variables(&dependency_instructions_list);
    }

    fn verify_environment_setter_called(
        environment_setter: &mut MockEnvironmentSetterTrait,
        dependency: &Dependency,
        json: &str,
    ) {
        environment_setter
            .expect_set_variable()
            .with(
                eq(dependency.clone()),
                eq(serde_json::from_str::<EnvironmentVariable>(json).unwrap()),
            )
            .times(1)
            .return_const(());
    }
}
