#[cfg(test)]
use mockall::automock;

use async_trait::async_trait;
use std::sync::Arc;

use futures::future::join_all;

use crate::solipath_instructions::data::dependency_instructions::DependencyInstructions;
use crate::solipath_platform::platform_filter::PlatformFilterTrait;
use crate::solipath_template::template_retriever::TemplateRetrieverTrait;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait LoopingTemplateRetrieverTrait {
    async fn retrieve_instructions_from_templates(
        &self,
        instructions_list: Vec<DependencyInstructions>,
    ) -> Vec<DependencyInstructions>;
}

pub struct LoopingTemplateRetriever {
    template_retriever: Arc<dyn TemplateRetrieverTrait + Sync + Send>,
    platform_filter: Arc<dyn PlatformFilterTrait + Sync + Send>,
}

impl LoopingTemplateRetriever {
    pub fn new(
        template_retriever: Arc<dyn TemplateRetrieverTrait + Sync + Send>,
        platform_filter: Arc<dyn PlatformFilterTrait + Sync + Send>,
    ) -> Self {
        Self {
            template_retriever,
            platform_filter,
        }
    }
}

#[async_trait]
impl LoopingTemplateRetrieverTrait for LoopingTemplateRetriever {
    async fn retrieve_instructions_from_templates(
        &self,
        instructions_list: Vec<DependencyInstructions>,
    ) -> Vec<DependencyInstructions> {
        let functions = instructions_list
            .into_iter()
            .map(|instruction| {
                let dependency = instruction.get_dependency();
                instruction
                    .get_templates()
                    .into_iter()
                    .filter(|template| {
                        self.platform_filter
                            .current_platform_is_match(template.get_platform_filters())
                    })
                    .map(move |template| {
                        self.template_retriever
                            .retrieve_instructions_from_template(dependency.clone(), template)
                    })
            })
            .flatten();
        join_all(functions).await
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::solipath_dependency_metadata::dependency::Dependency;
    use crate::solipath_instructions::data::install_instructions::InstallInstructions;
    use crate::solipath_instructions::data::template::Template;
    use crate::solipath_platform::platform::Platform;
    use crate::solipath_platform::platform_filter::mock::verify_platform_filter;
    use crate::solipath_platform::platform_filter::MockPlatformFilterTrait;
    use crate::solipath_template::template_retriever::MockTemplateRetrieverTrait;
    use mockall::predicate::eq;

    #[tokio::test]
    async fn single_template_retriever() {
        let dependency = Dependency::new("java", "11");
        let install_instructions = serde_json::from_str::<InstallInstructions>(
            r#"{"templates": [{"name": "template1", "variables": {"key1": "value1", "key2": "value2"}}]}"#,
        )
        .unwrap();
        let dependency_instructions = DependencyInstructions::new(dependency.clone(), install_instructions.clone());
        let output_install_instructions = serde_json::from_str::<InstallInstructions>(
            r#"{"environment_variables": [{"name": "name1", "relative_path": "path/something"}]}"#,
        )
        .unwrap();
        let output_dependency_instructions =
            DependencyInstructions::new(dependency.clone(), output_install_instructions.clone());

        let instructions_list = vec![dependency_instructions];
        let mut platform_filter = MockPlatformFilterTrait::new();
        verify_platform_filter(&mut platform_filter, Vec::new(), true, 1);

        let mut template_retriever = MockTemplateRetrieverTrait::new();
        template_retriever
            .expect_retrieve_instructions_from_template()
            .with(
                eq(dependency.clone()),
                eq(serde_json::from_str::<Template>(
                    r#"{"name": "template1", "variables": {"key1": "value1", "key2": "value2"}}"#,
                )
                .unwrap()),
            )
            .times(1)
            .return_const(output_dependency_instructions.clone());

        let looping_template_retriever =
            LoopingTemplateRetriever::new(Arc::new(template_retriever), Arc::new(platform_filter));

        let actual = looping_template_retriever
            .retrieve_instructions_from_templates(instructions_list)
            .await;
        assert_eq!(actual, vec!(output_dependency_instructions.clone()));
    }

    #[tokio::test]
    async fn multiple_template_retriever() {
        let dependency = Dependency::new("java", "11");
        let install_instructions = serde_json::from_str::<InstallInstructions>(
            r#"{"templates": [
                {"name": "template1", "variables": {"key1": "value1", "key2": "value2"}},
                {"name": "template2", "variables": {"key1": "value2", "key2": "value3"}}
            ]}"#,
        )
        .unwrap();
        let dependency_instructions = DependencyInstructions::new(dependency.clone(), install_instructions.clone());
        let output_install_instructions = serde_json::from_str::<InstallInstructions>(
            r#"{"environment_variables": [{"name": "name1", "relative_path": "path/something"}]}"#,
        )
        .unwrap();
        let output_install_instructions2 = serde_json::from_str::<InstallInstructions>(
            r#"{"environment_variables": [{"name": "name2", "relative_path": "path/something2"}]}"#,
        )
        .unwrap();
        let output_dependency_instructions =
            DependencyInstructions::new(dependency.clone(), output_install_instructions.clone());
        let output_dependency_instructions2 =
            DependencyInstructions::new(dependency.clone(), output_install_instructions2.clone());

        let instructions_list = vec![dependency_instructions];
        let mut platform_filter = MockPlatformFilterTrait::new();
        verify_platform_filter(&mut platform_filter, Vec::new(), true, 2);

        let mut template_retriever = MockTemplateRetrieverTrait::new();
        template_retriever
            .expect_retrieve_instructions_from_template()
            .with(
                eq(dependency.clone()),
                eq(serde_json::from_str::<Template>(
                    r#"{"name": "template1", "variables": {"key1": "value1", "key2": "value2"}}"#,
                )
                .unwrap()),
            )
            .times(1)
            .return_const(output_dependency_instructions.clone());
        template_retriever
            .expect_retrieve_instructions_from_template()
            .with(
                eq(dependency.clone()),
                eq(serde_json::from_str::<Template>(
                    r#"{"name": "template2", "variables": {"key1": "value2", "key2": "value3"}}"#,
                )
                .unwrap()),
            )
            .times(1)
            .return_const(output_dependency_instructions2.clone());

        let looping_template_retriever =
            LoopingTemplateRetriever::new(Arc::new(template_retriever), Arc::new(platform_filter));

        let actual = looping_template_retriever
            .retrieve_instructions_from_templates(instructions_list)
            .await;
        assert_eq!(
            actual,
            vec!(
                output_dependency_instructions.clone(),
                output_dependency_instructions2.clone()
            )
        );
    }

    #[tokio::test]
    async fn multiple_template_retriever_different_instructions() {
        let dependency = Dependency::new("java", "11");
        let install_instructions = serde_json::from_str::<InstallInstructions>(
            r#"{"templates": [
                {"name": "template1", "variables": {"key1": "value1", "key2": "value2"}}
            ]}"#,
        )
        .unwrap();
        let install_instructions2 = serde_json::from_str::<InstallInstructions>(
            r#"{"templates": [
                {"name": "template2", "variables": {"key1": "value2", "key2": "value3"}}
            ]}"#,
        )
        .unwrap();
        let dependency_instructions = DependencyInstructions::new(dependency.clone(), install_instructions.clone());
        let dependency_instructions2 = DependencyInstructions::new(dependency.clone(), install_instructions2.clone());
        let output_install_instructions = serde_json::from_str::<InstallInstructions>(
            r#"{"environment_variables": [{"name": "name1", "relative_path": "path/something"}]}"#,
        )
        .unwrap();
        let output_install_instructions2 = serde_json::from_str::<InstallInstructions>(
            r#"{"environment_variables": [{"name": "name2", "relative_path": "path/something2"}]}"#,
        )
        .unwrap();
        let output_dependency_instructions =
            DependencyInstructions::new(dependency.clone(), output_install_instructions.clone());
        let output_dependency_instructions2 =
            DependencyInstructions::new(dependency.clone(), output_install_instructions2.clone());

        let instructions_list = vec![dependency_instructions, dependency_instructions2];
        let mut platform_filter = MockPlatformFilterTrait::new();
        verify_platform_filter(&mut platform_filter, Vec::new(), true, 2);

        let mut template_retriever = MockTemplateRetrieverTrait::new();
        template_retriever
            .expect_retrieve_instructions_from_template()
            .with(
                eq(dependency.clone()),
                eq(serde_json::from_str::<Template>(
                    r#"{"name": "template1", "variables": {"key1": "value1", "key2": "value2"}}"#,
                )
                .unwrap()),
            )
            .times(1)
            .return_const(output_dependency_instructions.clone());
        template_retriever
            .expect_retrieve_instructions_from_template()
            .with(
                eq(dependency.clone()),
                eq(serde_json::from_str::<Template>(
                    r#"{"name": "template2", "variables": {"key1": "value2", "key2": "value3"}}"#,
                )
                .unwrap()),
            )
            .times(1)
            .return_const(output_dependency_instructions2.clone());

        let looping_template_retriever =
            LoopingTemplateRetriever::new(Arc::new(template_retriever), Arc::new(platform_filter));

        let actual = looping_template_retriever
            .retrieve_instructions_from_templates(instructions_list)
            .await;
        assert_eq!(
            actual,
            vec!(
                output_dependency_instructions.clone(),
                output_dependency_instructions2.clone()
            )
        );
    }

    #[tokio::test]
    async fn filtered_template_retriever() {
        let dependency = Dependency::new("java", "11");
        let install_instructions = serde_json::from_str::<InstallInstructions>(
            r#"{"templates": [
                {"name": "template1", "variables": {"key1": "value1", "key2": "value2"}, "platform_filters": [{"os": "good os", "arch": "x86"}]},
                {"name": "template2", "variables": {"key1": "value2", "key2": "value3"}, "platform_filters": [{"os": "bad os", "arch": "arm"}]}
            ]}"#
        ).unwrap();
        let dependency_instructions = DependencyInstructions::new(dependency.clone(), install_instructions.clone());
        let output_install_instructions = serde_json::from_str::<InstallInstructions>(
            r#"{"environment_variables": [{"name": "name1", "relative_path": "path/something"}]}"#,
        )
        .unwrap();
        let output_dependency_instructions =
            DependencyInstructions::new(dependency.clone(), output_install_instructions.clone());

        let instructions_list = vec![dependency_instructions];
        let mut platform_filter = MockPlatformFilterTrait::new();
        verify_platform_filter(&mut platform_filter, vec![Platform::new("good os", "x86")], true, 1);
        verify_platform_filter(&mut platform_filter, vec![Platform::new("bad os", "arm")], false, 1);

        let mut template_retriever = MockTemplateRetrieverTrait::new();
        template_retriever
            .expect_retrieve_instructions_from_template()
            .with(
                eq(dependency.clone()),
                eq(serde_json::from_str::<Template>(r#"{"name": "template1", "variables": {"key1": "value1", "key2": "value2"}, "platform_filters": [{"os": "good os", "arch": "x86"}]}"#).unwrap())
            )
            .times(1)
            .return_const(output_dependency_instructions.clone());

        let looping_template_retriever =
            LoopingTemplateRetriever::new(Arc::new(template_retriever), Arc::new(platform_filter));

        let actual = looping_template_retriever
            .retrieve_instructions_from_templates(instructions_list)
            .await;
        assert_eq!(actual, vec!(output_dependency_instructions.clone()));
    }
}
