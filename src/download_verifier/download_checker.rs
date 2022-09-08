use async_trait::async_trait;

use solipath_lib::solipath_download::file_downloader::FileDownloaderTrait;
use std::path::Path;
use std::path::PathBuf;
use reqwest::Client;

pub struct DownloadChecker{
    reqwest_client: Client,
}

impl DownloadChecker {
    pub fn new()-> Self{
        Self{reqwest_client: Client::new()}
    }
}

#[async_trait]
impl FileDownloaderTrait for DownloadChecker {
    async fn download_file_to_directory(&self, url: &str, _: &Path) -> PathBuf{
        let failure_message = format!("url {} failed to return", url);
        let response = self.reqwest_client.head(url).send().await.expect(&failure_message);
        if !response.status().is_success() {
            panic!("{}", failure_message);
        }
        println!("{} validated!", url);
        PathBuf::new()
    }
    async fn download_file(&self, _: &str, _: &Path){

    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn if_url_exists_does_not_panic(){
        DownloadChecker::new().download_file_to_directory("https://raw.githubusercontent.com/Solipath/Solipath/main/LICENSE-MIT", &Path::new(".")).await;
    }

    #[tokio::test]
    #[should_panic(expected = "url https://raw.githubusercontent.com/Solipath/Solipath/main/nonexistent-file failed to return")]
    async fn url_does_not_exist_panic() {
        DownloadChecker::new().download_file_to_directory("https://raw.githubusercontent.com/Solipath/Solipath/main/nonexistent-file", &Path::new(".")).await;
    }
}