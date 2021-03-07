use async_trait::async_trait;
use std::{path::Path, sync::Arc};
use tempfile::tempdir;

#[cfg(test)]
use mockall::automock;

use crate::solipath_download::file_decompressor::FileDecompressorTrait;
use crate::solipath_download::file_downloader::FileDownloaderTrait;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait ConditionalFileDownloaderTrait {
    async fn download_file_if_not_exists(&self, url: &str, path_to_save_to: &Path);
    async fn download_and_decompress_file_if_directory_not_exists(&self, url: &str, directory_to_save_to: &Path);
}

pub struct ConditionalFileDownloader {
    file_downloader: Arc<dyn FileDownloaderTrait + Sync + Send>,
    file_decompressor: Arc<dyn FileDecompressorTrait + Sync + Send>,
}

impl ConditionalFileDownloader {
    pub fn new(
        file_downloader: Arc<dyn FileDownloaderTrait + Sync + Send>,
        file_decompressor: Arc<dyn FileDecompressorTrait + Sync + Send>,
    ) -> Self {
        Self {
            file_downloader,
            file_decompressor,
        }
    }
}

#[async_trait]
impl ConditionalFileDownloaderTrait for ConditionalFileDownloader {
    async fn download_file_if_not_exists(&self, url: &str, path_to_save_to: &Path) {
        if (!path_to_save_to.exists()) {
            self.file_downloader.download_file(url, path_to_save_to).await;
        }
    }
    async fn download_and_decompress_file_if_directory_not_exists(&self, url: &str, directory_to_save_to: &Path) {
        if (!directory_to_save_to.exists()) {
            let temp_dir = tempdir().unwrap().into_path();
            let downloaded_file = self.file_downloader.download_file_to_directory(url, &temp_dir).await;
            self.file_decompressor
                .decompress_file_to_directory(&downloaded_file, &directory_to_save_to);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use super::*;
    use crate::solipath_download::file_decompressor::MockFileDecompressorTrait;
    use crate::solipath_download::file_downloader::MockFileDownloaderTrait;
    use std::path::PathBuf;

    #[tokio::test]
    async fn calls_file_downloader_when_file_does_not_exist() {
        let url = "https://something.com/";
        let temp_dir = tempdir().unwrap();
        let mut path_to_save_to = temp_dir.path().to_path_buf();
        path_to_save_to.push("file_that_should_not_exist.txt");
        let copy_path_to_save_to = path_to_save_to.clone();
        let mut file_downloader = MockFileDownloaderTrait::new();
        file_downloader
            .expect_download_file()
            .withf(move |actual_url, actual_path| actual_url == url && actual_path == copy_path_to_save_to)
            .times(1)
            .return_const(());
        let file_decompressor = MockFileDecompressorTrait::new();
        let conditional_file_downloader =
            ConditionalFileDownloader::new(Arc::new(file_downloader), Arc::new(file_decompressor));

        conditional_file_downloader
            .download_file_if_not_exists(url, &path_to_save_to.clone())
            .await;
    }

    #[tokio::test]
    async fn does_not_call_download_file_if_file_exists() {
        let url = "https://something.com";
        let path_to_save_to = tempdir().unwrap();
        let mut path = path_to_save_to.path().to_path_buf();
        path.push("randomfile.txt");
        File::create(path.clone()).expect("failed to create tempfile");
        let mut file_downloader = MockFileDownloaderTrait::new();
        file_downloader.expect_download_file().times(0).return_const(());
        let file_decompressor = MockFileDecompressorTrait::new();
        let conditional_file_downloader =
            ConditionalFileDownloader::new(Arc::new(file_downloader), Arc::new(file_decompressor));

        conditional_file_downloader
            .download_file_if_not_exists(url, &path)
            .await;
    }

    #[tokio::test]
    async fn calls_download_file_and_decompress_file_if_directory_does_not_exist() {
        let url = "https://something.com/download.zip";
        let temp_dir = tempdir().unwrap();
        let mut path_to_save_to = temp_dir.path().to_path_buf();
        path_to_save_to.push("directory_that_should_not_exist");
        let copy_path_to_save_to = path_to_save_to.clone();
        
        let mut file_downloader = MockFileDownloaderTrait::new();
        file_downloader
            .expect_download_file_to_directory()
            .withf(move |actual_url, _| actual_url == url)
            .times(1)
            .return_const(PathBuf::from("/output_path/download.zip"));
        let mut file_decompressor = MockFileDecompressorTrait::new();
        file_decompressor
            .expect_decompress_file_to_directory()
            .withf(move |source_file_path, target_directory| {
                source_file_path == PathBuf::from("/output_path/download.zip") && target_directory == copy_path_to_save_to
            })
            .times(1)
            .return_const(());
        let conditional_file_downloader =
            ConditionalFileDownloader::new(Arc::new(file_downloader), Arc::new(file_decompressor));
        conditional_file_downloader
            .download_and_decompress_file_if_directory_not_exists(url, &path_to_save_to)
            .await;
    }

    #[tokio::test]
    async fn does_not_call_download_file_if_directory_exists() {
        let url = "https://something.com/download.zip";
        let path_to_save_to = tempdir().unwrap().into_path();

        let mut file_downloader = MockFileDownloaderTrait::new();
        file_downloader.expect_download_file().times(0).return_const(());
        let mut file_decompressor = MockFileDecompressorTrait::new();
        file_decompressor
            .expect_decompress_file_to_directory()
            .times(0)
            .return_const(());
        let conditional_file_downloader =
            ConditionalFileDownloader::new(Arc::new(file_downloader), Arc::new(file_decompressor));
        conditional_file_downloader
            .download_and_decompress_file_if_directory_not_exists(url, &path_to_save_to)
            .await;
    }
}
