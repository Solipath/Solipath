use async_trait::async_trait;
use std::sync::Arc;

#[cfg(test)]
use mockall::automock;

use crate::solipath_dependency_metadata::dependency::Dependency;
use crate::solipath_directory::solipath_directory_finder::SolipathDirectoryFinderTrait;
use crate::solipath_download::conditional_file_downloader::ConditionalFileDownloaderTrait;
use crate::solipath_instructions::data::download_instruction::DownloadInstruction;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait DependencyDownloaderTrait {
    async fn download_dependency(&self, dependency: Dependency, download_instruction: DownloadInstruction);
}

pub struct DependencyDownloader {
    directory_finder: Arc<dyn SolipathDirectoryFinderTrait + Send + Sync>,
    conditional_file_downloader: Arc<dyn ConditionalFileDownloaderTrait + Send + Sync>,
}

impl DependencyDownloader {
    pub fn new(
        directory_finder: Arc<dyn SolipathDirectoryFinderTrait + Send + Sync>,
        conditional_file_downloader: Arc<dyn ConditionalFileDownloaderTrait + Send + Sync>,
    ) -> Self {
        Self {
            directory_finder,
            conditional_file_downloader,
        }
    }
}

#[async_trait]
impl DependencyDownloaderTrait for DependencyDownloader {
    async fn download_dependency(&self, dependency: Dependency, download_instruction: DownloadInstruction) {
        let mut downloads_directory = self.directory_finder.get_dependency_downloads_directory(&dependency);
        downloads_directory.push(download_instruction.get_destination_directory());
        self.conditional_file_downloader
            .download_and_decompress_file_if_directory_not_exists(&download_instruction.get_url(), &downloads_directory)
            .await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solipath_directory::solipath_directory_finder::MockSolipathDirectoryFinderTrait;
    use crate::solipath_download::conditional_file_downloader::MockConditionalFileDownloaderTrait;
    use mockall::predicate::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn can_download_dependency() {
        let dependency = Dependency::new("Java", "11");
        let download_instruction: DownloadInstruction =
            serde_json::from_str(r#"{"url": "www.github.com/download.zip", "destination_directory": "destination"}"#)
                .unwrap();
        let downloads_directory = PathBuf::from("downloads/directory");
        let mut directory_finder = MockSolipathDirectoryFinderTrait::new();
        directory_finder
            .expect_get_dependency_downloads_directory()
            .with(eq(dependency.clone()))
            .times(1)
            .return_const(downloads_directory);
        let mut conditional_file_downloader = MockConditionalFileDownloaderTrait::new();
        conditional_file_downloader
            .expect_download_and_decompress_file_if_directory_not_exists()
            .withf(|actual_url, actual_path| {
                actual_url == "www.github.com/download.zip"
                    && actual_path == PathBuf::from("downloads/directory/destination")
            })
            .times(1)
            .return_const(());
        let dependency_downloader =
            DependencyDownloader::new(Arc::new(directory_finder), Arc::new(conditional_file_downloader));
        dependency_downloader
            .download_dependency(dependency, download_instruction)
            .await;
    }
}
