use std::{path::Path, sync::Arc};

use async_trait::async_trait;
use tokio::fs::read_to_string;

use crate::solipath_download::conditional_file_downloader::ConditionalFileDownloaderTrait;

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait FileToStringDownloaderTrait {
    async fn download_file_then_parse_to_string(&self, url: &str, path_to_save_to: &Path) -> String;
}

pub struct FileToStringDownloader {
    conditional_file_downloader: Arc<dyn ConditionalFileDownloaderTrait + Sync + Send>,
}

impl FileToStringDownloader {
    pub fn new(conditional_file_downloader: Arc<dyn ConditionalFileDownloaderTrait + Sync + Send>) -> Self {
        Self {
            conditional_file_downloader,
        }
    }
}

#[async_trait]
impl FileToStringDownloaderTrait for FileToStringDownloader {
    async fn download_file_then_parse_to_string(&self, url: &str, path_to_save_to: &Path) -> String {
        self.conditional_file_downloader
            .download_file_if_not_exists(url, path_to_save_to)
            .await;
        read_to_string(path_to_save_to).await.expect("failed to read file")
    }
}

#[cfg(test)]
mod test {

    use tempfile::tempdir;

    use super::*;
    use crate::solipath_download::conditional_file_downloader::MockConditionalFileDownloaderTrait;
    use tokio::{fs::File, io::AsyncWriteExt};

    #[tokio::test]
    async fn can_read_file_after_download() {
        let temp_dir = tempdir().unwrap();
        let dependency_directory = temp_dir.into_path().to_path_buf();
        let mut path_to_downloaded_file = dependency_directory.clone();
        path_to_downloaded_file.push("install_instructions.json");
        let copy_path_to_downloaded_file = path_to_downloaded_file.clone();
        let mut file = File::create(path_to_downloaded_file.clone()).await.unwrap();
        file.write_all("the file contents".as_bytes())
            .await
            .expect("failed to write to file");
        let passed_in_url = "http://www.github.com/name/version/install_instructions.json";
        let mut mock_file_downloader = MockConditionalFileDownloaderTrait::new();
        mock_file_downloader
            .expect_download_file_if_not_exists()
            .withf(move |url, path| url == passed_in_url && path == copy_path_to_downloaded_file)
            .times(1)
            .return_const(());

        let file_retriever = FileToStringDownloader::new(Arc::new(mock_file_downloader));
        let actual = file_retriever
            .download_file_then_parse_to_string(passed_in_url, &path_to_downloaded_file.clone())
            .await;
        assert_eq!(actual, "the file contents");
    }
}
