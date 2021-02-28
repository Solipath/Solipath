use async_trait::async_trait;
use futures::future::join_all;
use std::sync::Arc;

#[cfg(test)]
use mockall::automock;

use crate::solipath_dependency_download::dependency_downloader::DependencyDownloaderTrait;
use crate::{
    solipath_instructions::data::dependency_instructions::DependencyInstructions,
    solipath_platform::platform_filter::PlatformFilterTrait,
};

#[cfg_attr(test, automock)]
#[async_trait]
pub trait LoopingDependencyDownloaderTrait {
    async fn download_dependencies(&self, dependency_instructions_list: Vec<DependencyInstructions>);
}

pub struct LoopingDependencyDownloader {
    dependency_downloader: Arc<dyn DependencyDownloaderTrait + Send + Sync>,
    platform_filter: Arc<dyn PlatformFilterTrait + Send + Sync>,
}

impl LoopingDependencyDownloader {
    pub fn new(
        dependency_downloader: Arc<dyn DependencyDownloaderTrait + Send + Sync>,
        platform_filter: Arc<dyn PlatformFilterTrait + Send + Sync>,
    ) -> Self {
        Self {
            dependency_downloader,
            platform_filter,
        }
    }
}

#[async_trait]
impl LoopingDependencyDownloaderTrait for LoopingDependencyDownloader {
    async fn download_dependencies(&self, dependency_instructions_list: Vec<DependencyInstructions>) {
        let download_tasks = dependency_instructions_list
            .into_iter()
            .map(|dependency_instructions| {
                dependency_instructions
                    .get_downloads()
                    .into_iter()
                    .filter(|download| {
                        self.platform_filter
                            .current_platform_is_match(download.get_platform_filters())
                    })
                    .map(move |download| {
                        self.dependency_downloader
                            .download_dependency(dependency_instructions.get_dependency(), download)
                    })
            })
            .flatten();
        join_all(download_tasks).await;
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::solipath_dependency_download::dependency_downloader::MockDependencyDownloaderTrait;
    use crate::solipath_dependency_metadata::dependency::Dependency;
    use crate::solipath_instructions::data::download_instruction::DownloadInstruction;
    use crate::solipath_instructions::data::install_instructions::InstallInstructions;
    use crate::solipath_platform::platform::Platform;
    use crate::solipath_platform::platform_filter::mock::verify_platform_filter;
    use crate::solipath_platform::platform_filter::MockPlatformFilterTrait;
    use mockall::predicate::*;

    #[tokio::test]
    async fn download_dependencies_single_download() {
        let node = Dependency::new("Node", "15");
        let node_dependency_instructions = DependencyInstructions::new(
            node.clone(),
            serde_json::from_str::<InstallInstructions>(
                r#"
        {"downloads": [
            {"url": "www.github.com/node15.zip", "destination_directory": "node15Folder", "platform_filters": [{"os": "a good match", "arch": "x86"}]}
        ]}"#,
            )
            .unwrap(),
        );
        let dependency_instructions_list: Vec<DependencyInstructions> = vec![node_dependency_instructions.clone()];

        let mut platform_filter = MockPlatformFilterTrait::new();
        verify_platform_filter(
            &mut platform_filter,
            vec![Platform::new("a good match", "x86")],
            true,
            1,
        );

        let mut dependency_downloader = MockDependencyDownloaderTrait::new();
        verify_download_dependency_called(
            &mut dependency_downloader,
            &node,
            r#"{
            "url": "www.github.com/node15.zip", "destination_directory": "node15Folder", 
            "platform_filters": [{"os": "a good match", "arch": "x86"}]
        }"#,
        );

        let looping_dependency_downloader =
            LoopingDependencyDownloader::new(Arc::new(dependency_downloader), Arc::new(platform_filter));
        looping_dependency_downloader
            .download_dependencies(dependency_instructions_list)
            .await;
    }

    #[tokio::test]
    async fn download_dependencies_two_downloads_one_is_filtered_out() {
        let node = Dependency::new("Node", "15");
        let node_dependency_instructions = DependencyInstructions::new(node.clone(), serde_json::from_str::<InstallInstructions>(r#"
        {"downloads": [
            {"url": "www.github.com/node15.zip", "destination_directory": "node15Folder", "platform_filters": [{"os": "a good match", "arch": "x86"}]},
            {"url": "www.github.com/gradle.zip", "destination_directory": "gradleFolder", "platform_filters": [{"os": "a bad match", "arch": "arm"}]}
        ]}"#).unwrap());
        let dependency_instructions_list: Vec<DependencyInstructions> = vec![node_dependency_instructions.clone()];

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

        let mut dependency_downloader = MockDependencyDownloaderTrait::new();
        verify_download_dependency_called(
            &mut dependency_downloader,
            &node,
            r#"{
            "url": "www.github.com/node15.zip", "destination_directory": "node15Folder", 
            "platform_filters": [{"os": "a good match", "arch": "x86"}]
        }"#,
        );

        let looping_dependency_downloader =
            LoopingDependencyDownloader::new(Arc::new(dependency_downloader), Arc::new(platform_filter));
        looping_dependency_downloader
            .download_dependencies(dependency_instructions_list)
            .await;
    }

    #[tokio::test]
    async fn download_dependencies_multiple_downloads() {
        let java = Dependency::new("Java", "14");
        let node = Dependency::new("Node", "15");
        let java_json = r#" 
        {"downloads": [
            {"url": "www.github.com/java-14.zip", "destination_directory": "java14folder"},
            {"url": "www.github.com/other_download.zip", "destination_directory": "other_java_download_folder"}
        ]}"#;
        let java_dependency_instructions = DependencyInstructions::new(
            java.clone(),
            serde_json::from_str::<InstallInstructions>(java_json).unwrap(),
        );
        let node_json = r#"
        {"downloads": [
            {"url": "www.github.com/node15.zip", "destination_directory": "node15Folder"}
        ]}"#;
        let node_dependency_instructions = DependencyInstructions::new(
            node.clone(),
            serde_json::from_str::<InstallInstructions>(node_json).unwrap(),
        );
        let dependency_instructions_list: Vec<DependencyInstructions> = vec![
            java_dependency_instructions.clone(),
            node_dependency_instructions.clone(),
        ];
        let mut platform_filter = MockPlatformFilterTrait::new();
        verify_platform_filter(&mut platform_filter, Vec::new(), true, 3);

        let mut dependency_downloader = MockDependencyDownloaderTrait::new();
        verify_download_dependency_called(
            &mut dependency_downloader,
            &java,
            r#"{"url": "www.github.com/java-14.zip", "destination_directory": "java14folder"}"#,
        );
        verify_download_dependency_called(
            &mut dependency_downloader,
            &java,
            r#"{"url": "www.github.com/other_download.zip", "destination_directory": "other_java_download_folder"}"#,
        );
        verify_download_dependency_called(
            &mut dependency_downloader,
            &node,
            r#"{"url": "www.github.com/node15.zip", "destination_directory": "node15Folder"}"#,
        );

        let looping_dependency_downloader =
            LoopingDependencyDownloader::new(Arc::new(dependency_downloader), Arc::new(platform_filter));
        looping_dependency_downloader
            .download_dependencies(dependency_instructions_list)
            .await;
    }

    fn verify_download_dependency_called(
        dependency_downloader: &mut MockDependencyDownloaderTrait,
        dependency: &Dependency,
        json: &str,
    ) {
        dependency_downloader
            .expect_download_dependency()
            .with(
                eq(dependency.clone()),
                eq(serde_json::from_str::<DownloadInstruction>(json).unwrap()),
            )
            .times(1)
            .return_const(());
    }
}
