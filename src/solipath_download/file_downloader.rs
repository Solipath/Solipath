use anyhow::Context;
use async_trait::async_trait;
use reqwest::Client;
use reqwest::Error;
use reqwest::Response;
use tokio::time::sleep;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;
use tokio::{
    fs::{create_dir_all, File},
    io::AsyncWriteExt,
};
use anyhow::Result;
use crate::solipath_download::file_name_retriever::*;

#[cfg(test)]
use mockall::{automock, predicate::*};

#[cfg_attr(test, automock)]
#[async_trait]
pub trait FileDownloaderTrait {
    async fn download_file_to_directory(&self, url: &str, directory_to_save_to: &Path) -> Result<PathBuf>;
    async fn download_file(&self, url: &str, path: &Path);
}

pub struct FileDownloader {
    reqwest_client: Client,
}

impl FileDownloader {
    pub fn new() -> Self {
        Self {
            reqwest_client: Client::new(),
        }
    }

    async fn repeat_request(&self, url: &str) -> Result<Response> {
        let mut number_of_tries = 0;
        let max_number_of_tries = 3;
        let mut result = self.make_request(url).await;
        while result.is_err() && number_of_tries < max_number_of_tries{
            println!("Error downloading {}, trying again...", url);
            result = self.make_request(url).await;
            number_of_tries += 1;
            sleep(Duration::new(2*number_of_tries, 0)).await
        }
        Ok(result.context(format!("failed to download file: {}", url))?)
    }

    async fn make_request(&self, url: &str) -> Result<Response, Error> {
        self.reqwest_client
                    .get(url)
                    .send()
                    .await.and_then(|response| response.error_for_status())
    }

    async fn stream_response_output_to_file(&self, response: &mut Response, file: &mut File) {
        while let Some(chunk) = response.chunk().await.expect("file download failed!") {
            file.write_all(&chunk)
                .await
                .expect("failed to write to file as part of download");
        }
    }
}


#[async_trait]
impl FileDownloaderTrait for FileDownloader {
    async fn download_file_to_directory(&self, url: &str, directory_to_save_to: &Path) -> Result<PathBuf> {
        println!("downloading {}...", url);
        let mut response = self.repeat_request(url).await?;
        create_dir_all(&directory_to_save_to)
            .await
            .with_context(||"failed to create directory")?;
        let file_name = get_file_name(response.url().as_str(), response.headers());
        let mut path_to_save_to = directory_to_save_to.to_path_buf();
        path_to_save_to.push(file_name.clone());
        let mut file = File::create(path_to_save_to.clone())
            .await
            .context(format!("could not create file: {}", file_name))?;
        self.stream_response_output_to_file(&mut response, &mut file).await;
        println!("finished downloading {}", url);
        Ok(path_to_save_to)
    }

    async fn download_file(&self, url: &str, path_to_save_to: &Path) {
        println!("downloading {}...", url);
        let mut response = self.repeat_request(url).await.unwrap();
        let parent_directory = path_to_save_to.parent().unwrap();
        create_dir_all(&parent_directory)
            .await
            .expect("failed to create parent directories");
        let mut file = File::create(path_to_save_to).await.expect("could not create file");
        self.stream_response_output_to_file(&mut response, &mut file).await;
        println!("finished downloading {}", url);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::fs::read_to_string;

    const DOWNLOAD_CONTENT: &str = r#"Permission is hereby granted, free of charge, to any
person obtaining a copy of this software and associated
documentation files (the "Software"), to deal in the
Software without restriction, including without
limitation the rights to use, copy, modify, merge,
publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software
is furnished to do so, subject to the following
conditions:

The above copyright notice and this permission notice
shall be included in all copies or substantial portions
of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
DEALINGS IN THE SOFTWARE.
"#;

    #[tokio::test]
    async fn can_download_a_file() {
        let temp_dir = tempdir().unwrap().into_path();
        let expected_file_path = temp_dir.join("LICENSE-MIT".to_string());
        let file_downloader = FileDownloader::new();

        file_downloader
            .download_file(
                "https://raw.githubusercontent.com/rust-lang/rust/master/LICENSE-MIT",
                &expected_file_path,
            )
            .await;

        let file_contents = read_to_string(expected_file_path.to_str().unwrap())
            .await
            .expect("something went wrong trying to read file");

        assert_eq!(file_contents, DOWNLOAD_CONTENT);
    }

    #[tokio::test]
    #[should_panic(expected="failed to download file: https://raw.githubusercontent.com/rust-lang/rust/master/NONEXISTENT_FILE")]
    async fn a_404_throws_an_exception(){
        let file_downloader = FileDownloader::new();
        file_downloader.repeat_request("https://raw.githubusercontent.com/rust-lang/rust/master/NONEXISTENT_FILE").await.unwrap();
    }



    #[tokio::test]
    async fn can_download_a_file_to_directory() {
        let temp_dir = tempdir().unwrap().into_path();
        let file_downloader = FileDownloader::new();

        let actual_path = file_downloader
            .download_file_to_directory(
                "https://raw.githubusercontent.com/rust-lang/rust/master/LICENSE-MIT",
                &temp_dir,
            )
            .await.expect("something went wrong when downloadingi file");

        let file_contents = read_to_string(actual_path.to_str().unwrap())
            .await
            .expect("something went wrong trying to read file");

        assert_eq!(file_contents, DOWNLOAD_CONTENT);
    }
}
