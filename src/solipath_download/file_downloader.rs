use async_trait::async_trait;
use reqwest::Client;
use std::path::Path;
use tokio::{
    fs::{create_dir_all, File},
    io::AsyncWriteExt,
};

#[cfg(test)]
use mockall::{automock, predicate::*};

#[cfg_attr(test, automock)]
#[async_trait]
pub trait FileDownloaderTrait {
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
}

#[async_trait]
impl FileDownloaderTrait for FileDownloader {
    async fn download_file(&self, url: &str, path_to_save_to: &Path) {
        let mut response = self
            .reqwest_client
            .get(url)
            .send()
            .await
            .expect("file download failed!");
        let parent_directory = path_to_save_to.parent().unwrap();
        create_dir_all(&parent_directory)
            .await
            .expect("failed to create parent directories");
        let mut file = File::create(path_to_save_to.clone())
            .await
            .expect("could not create file");
        println!("downloading {}...", url);
        while let Some(chunk) = response.chunk().await.expect("file download failed!") {
            file.write_all(&chunk)
                .await
                .expect("failed to write to file as part of download");
        }
        println!("completed downloading {}", url);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::fs::read_to_string;

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

        let expected = r#"Permission is hereby granted, free of charge, to any
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
        assert_eq!(file_contents, expected);
    }
}
