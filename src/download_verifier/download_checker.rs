use anyhow::Context;
use async_trait::async_trait;

use reqwest::Response;
use solipath_lib::solipath_download::file_downloader::FileDownloaderTrait;
use tokio::time::sleep;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;
use reqwest::Client;
use anyhow::Result;

pub struct DownloadChecker{
    reqwest_client: Client,
}

impl DownloadChecker {
    pub fn new()-> Self{
        Self{reqwest_client: Client::new()}
    }
    async fn repeat_request(&self, url: &str) -> Result<Response> {
        let mut number_of_tries = 0;
        let max_number_of_tries = 3;
        let mut result = self.reqwest_client.head(url).send().await;
        while result.is_err() && number_of_tries < max_number_of_tries{
            result = self.reqwest_client.head(url).send().await;
            number_of_tries += 1;
            sleep(Duration::new(1u64, 0)).await
        }
        Ok(result.context(format!("failed to download file: {}", url))?)
    }
}

#[async_trait]
impl FileDownloaderTrait for DownloadChecker {
    async fn download_file_to_directory(&self, url: &str, _: &Path) -> Result<PathBuf>{
        
        let failure_message = format!("url {} failed to return", url);
        let response = self.repeat_request(url).await?;
        if !response.status().is_success() {
            panic!("{}", failure_message);
        }
        println!("{} validated!", url);
        Ok(PathBuf::new())
    }
    async fn download_file(&self, _: &str, _: &Path){

    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn if_url_exists_does_not_panic(){
        DownloadChecker::new().download_file_to_directory("https://raw.githubusercontent.com/Solipath/Solipath/main/LICENSE-MIT", &Path::new(".")).await.unwrap();
    }

    #[tokio::test]
    #[should_panic(expected = "url https://raw.githubusercontent.com/Solipath/Solipath/main/nonexistent-file failed to return")]
    async fn url_does_not_exist_panic() {
        DownloadChecker::new().download_file_to_directory("https://raw.githubusercontent.com/Solipath/Solipath/main/nonexistent-file", &Path::new(".")).await.unwrap();
    }
}