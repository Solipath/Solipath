use std::sync::Arc;

#[cfg(test)]
use mockall::automock;

use async_trait::async_trait;

use crate::solipath_dependency_metadata::dependency::Dependency;
use crate::solipath_instructions::data::dependency_instructions::DependencyInstructions;
use crate::solipath_instructions::dependency_instructions_retriever::DependencyInstructionsRetrieverTrait;
use crate::solipath_platform::platform_filter::{run_functions_matching_platform, PlatformFilterTrait};

#[cfg_attr(test, automock)]
#[async_trait]
pub trait LoopingDependencyInstructionsRetrieverTrait {
    async fn retrieve_dependency_instructions_list(
        &self,
        dependency_list: Vec<Dependency>,
    ) -> Vec<DependencyInstructions>;
}

pub struct LoopingDependencyInstructionsRetriever {
    dependency_instructions_retriever: Arc<dyn DependencyInstructionsRetrieverTrait + Sync + Send>,
    platform_filter: Arc<dyn PlatformFilterTrait + Sync + Send>,
}

impl LoopingDependencyInstructionsRetriever {
    pub fn new(
        dependency_instructions_retriever: Arc<dyn DependencyInstructionsRetrieverTrait + Sync + Send>,
        platform_filter: Arc<dyn PlatformFilterTrait + Sync + Send>,
    ) -> Self {
        Self {
            dependency_instructions_retriever,
            platform_filter,
        }
    }
}

#[async_trait]
impl LoopingDependencyInstructionsRetrieverTrait for LoopingDependencyInstructionsRetriever {
    async fn retrieve_dependency_instructions_list(
        &self,
        dependency_list: Vec<Dependency>,
    ) -> Vec<DependencyInstructions> {
        run_functions_matching_platform(&self.platform_filter, &dependency_list, |dependency| {
            self.dependency_instructions_retriever
                .retrieve_dependency_instructions(dependency)
        }).await
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::solipath_instructions::dependency_instructions_retriever::MockDependencyInstructionsRetrieverTrait;
    use crate::solipath_platform::platform::Platform;
    use crate::solipath_platform::platform_filter::mock::verify_platform_filter;
    use crate::solipath_platform::platform_filter::MockPlatformFilterTrait;
    use mockall::predicate::*;
    use std::vec::Vec;

    #[tokio::test]
    async fn loops_through_each_dependency_calls_retrieve_dependency_instructions_for_each() {
        let dependency1 = Dependency::new("name1", "version1");
        let dependency2 = Dependency::new("name2", "version2");
        let dependency_list = vec![dependency1.clone(), dependency2.clone()];
        let dependency_instruction1 =
            DependencyInstructions::new(dependency1.clone(), serde_json::from_str("{}").unwrap());
        let dependency_instruction2 =
            DependencyInstructions::new(dependency2.clone(), serde_json::from_str("{}").unwrap());
        let expected = vec![dependency_instruction1.clone(), dependency_instruction2.clone()];
        let mut file_retriever = MockDependencyInstructionsRetrieverTrait::new();
        file_retriever
            .expect_retrieve_dependency_instructions()
            .with(eq(dependency1.clone()))
            .times(1)
            .return_const(dependency_instruction1.clone());
        file_retriever
            .expect_retrieve_dependency_instructions()
            .with(eq(dependency2.clone()))
            .times(1)
            .return_const(dependency_instruction2.clone());
        let mut platform_filter = MockPlatformFilterTrait::new();
        verify_platform_filter(&mut platform_filter, Vec::new(), true, 2);
        let list_retriever =
            LoopingDependencyInstructionsRetriever::new(Arc::new(file_retriever), Arc::new(platform_filter));

        let actual = list_retriever
            .retrieve_dependency_instructions_list(dependency_list)
            .await;

        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn loops_through_and_filters_out_dependency_if_not_match() {
        let dependency1: Dependency = serde_json::from_str(
            r#"{"name": "name1", "version": "version1", "platform_filters": [{"os": "a good match", "arch": "x86"}]} "#,
        )
        .unwrap();
        let dependency2: Dependency = serde_json::from_str(
            r#"{"name": "name2", "version": "version2", "platform_filters": [{"os": "a bad match", "arch": "arm"}]} "#,
        )
        .unwrap();
        let dependency_list = vec![dependency1.clone(), dependency2.clone()];
        let dependency_instruction1 =
            DependencyInstructions::new(dependency1.clone(), serde_json::from_str("{}").unwrap());

        let expected = vec![dependency_instruction1.clone()];
        let mut file_retriever = MockDependencyInstructionsRetrieverTrait::new();
        file_retriever
            .expect_retrieve_dependency_instructions()
            .with(eq(dependency1.clone()))
            .times(1)
            .return_const(dependency_instruction1.clone());
        let mut platform_filter = MockPlatformFilterTrait::new();
        verify_platform_filter(
            &mut platform_filter,
            vec![Platform::new("a good match", "x86")],
            true,
            1,
        );
        verify_platform_filter(
            &mut platform_filter,
            vec![Platform::new("a bad match", "arm")],
            false,
            1,
        );
        let list_retriever =
            LoopingDependencyInstructionsRetriever::new(Arc::new(file_retriever), Arc::new(platform_filter));

        let actual = list_retriever
            .retrieve_dependency_instructions_list(dependency_list)
            .await;

        assert_eq!(actual, expected);
    }
}
