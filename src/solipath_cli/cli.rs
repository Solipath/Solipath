use std::{fs, path::Path, sync::Arc};

use crate::{
    solipath_directory::solipath_directory_finder::{SolipathDirectoryFinder, SolipathDirectoryFinderTrait},
    solipath_download::file_downloader::{FileDownloader, FileDownloaderTrait},
    solipath_platform::{
        current_platform_retriever::{CurrentPlatformRetriever, CurrentPlatformRetrieverTrait},
        platform::Platform,
    },
};
use anyhow::{Context, Result};

pub fn is_solipath_command(commands: &[String]) -> bool {
    commands[0].starts_with("--")
}
pub struct SolipathCli {
    file_downloader: Arc<dyn FileDownloaderTrait + Sync + Send>,
    directory_finder: Arc<dyn SolipathDirectoryFinderTrait + Sync + Send>,
    current_platform_retriever: Arc<dyn CurrentPlatformRetrieverTrait + Sync + Send>,
}
impl SolipathCli {
    pub fn new() -> Self {
        Self {
            file_downloader: Arc::new(FileDownloader::new()),
            directory_finder: Arc::new(SolipathDirectoryFinder::new()),
            current_platform_retriever: Arc::new(CurrentPlatformRetriever::new()),
        }
    }
    pub async fn run_solipath_command(&self, commands: &[String])-> Result<()> {
        match commands[0].as_str() {
            "--update" => self.update_solipath().await,
            _ => {Ok(())}
        }
    }

    async fn update_solipath(&self)-> Result<()> {
        let Platform { os, arch } = self.current_platform_retriever.get_current_platform();
        let solipath_directory = self.directory_finder.get_base_solipath_directory();
        let file_extension = get_executable_file_extension(&os);
        let solipath_url = format!(
            "https://github.com/Solipath/Solipath/releases/download/latest-{}_{}/solipath{}",
            os,
            arch.unwrap(),
            file_extension
        );
        let mut original_executable = solipath_directory.clone();
        original_executable.push(format!("solipath{}", file_extension));
        let mut renamed_executable = solipath_directory.clone();
        renamed_executable.push(format!("solipathold{}", file_extension));
        fs::rename(&original_executable, &renamed_executable)
            .with_context(||"failed to move current solipath executable, this might mean a process is holding onto the file, or you don't have permission to move it.")?;
        self.file_downloader
            .download_file_to_directory(&solipath_url, &solipath_directory)
            .await
            .and_then(|_|{
                set_file_as_executable(&original_executable);
                Ok(())
            })
            .or_else(|error| {
                println!("failed to download solipath. Moving executable back to original location");
                fs::rename(&renamed_executable, &original_executable)?;
                Err(error)
            })
    }
}

#[cfg(not(target_os = "windows"))]
fn set_file_as_executable(executable_path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(executable_path, fs::Permissions::from_mode(0o775))
        .expect("failed to set solipath executable with execute permissions");
}
#[cfg(target_os = "windows")]
fn set_file_as_executable(executable_path: &Path) {}

fn get_executable_file_extension(os: &str) -> String {
    if os == "windows" {
        ".exe".to_string()
    } else {
        "".to_string()
    }
}

#[cfg(test)]
mod test {
    use std::{fs, path::PathBuf};

    use anyhow::Error;
    use mockall::predicate::eq;
    use tempfile::tempdir;

    use crate::{
        solipath_directory::solipath_directory_finder::MockSolipathDirectoryFinderTrait,
        solipath_download::file_downloader::MockFileDownloaderTrait,
        solipath_platform::current_platform_retriever::MockCurrentPlatformRetrieverTrait,
    };

    use super::*;

    #[test]
    fn is_solipath_command_returns_true_if_first_starts_with_dashes() {
        assert_eq!(is_solipath_command(&["--a-command".to_string()]), true);
    }

    #[test]
    fn is_solipath_command_returns_false_if_first_command_does_not_start_with_dashes() {
        assert_eq!(is_solipath_command(&["a-command".to_string()]), false);
    }

    #[tokio::test]
    async fn run_solipath_update_and_check_download_size() {
        let solipath_temp_dir = tempdir().unwrap().into_path();
        let mut solipath_executable = solipath_temp_dir.clone();
        solipath_executable.push(format!(
            "solipath{}",
            get_executable_file_extension(std::env::consts::OS)
        ));
        fs::write(&solipath_executable, "not a real executable").unwrap();
        let mut mock_directory_finder = MockSolipathDirectoryFinderTrait::new();
        mock_directory_finder
            .expect_get_base_solipath_directory()
            .once()
            .return_const(solipath_temp_dir.clone());

        let solipath_cli = SolipathCli {
            file_downloader: Arc::new(FileDownloader::new()),
            directory_finder: Arc::new(mock_directory_finder),
            current_platform_retriever: Arc::new(CurrentPlatformRetriever::new()),
        };
        solipath_cli.run_solipath_command(&["--update".to_string()]).await.unwrap();

        let file_metadata = solipath_executable.metadata().unwrap();
        assert!(file_metadata.len() > 5000000);
    }

    #[tokio::test]
    async fn invalid_command_does_nothing() {
        let solipath_temp_dir = tempdir().unwrap().into_path();
        let mock_directory_finder = MockSolipathDirectoryFinderTrait::new();

        let solipath_cli = SolipathCli {
            file_downloader: Arc::new(FileDownloader::new()),
            directory_finder: Arc::new(mock_directory_finder),
            current_platform_retriever: Arc::new(CurrentPlatformRetriever::new()),
        };
        solipath_cli
            .run_solipath_command(&["--not-real-command".to_string()])
            .await.unwrap();
        let mut directory = fs::read_dir(solipath_temp_dir.clone()).unwrap();
        assert!(directory.next().is_none())
    }

    #[tokio::test]
    async fn linux_downloads_solipath_without_extention() {
        let solipath_temp_dir = tempdir().unwrap().into_path();
        let mut fake_solipath_executable = solipath_temp_dir.clone();
        fake_solipath_executable.push("solipath");
        fs::write(&fake_solipath_executable, "not a real executable").unwrap();
        let mut mock_directory_finder = MockSolipathDirectoryFinderTrait::new();
        mock_directory_finder
            .expect_get_base_solipath_directory()
            .once()
            .return_const(solipath_temp_dir.clone());
        let mut mock_current_platform_retriever = MockCurrentPlatformRetrieverTrait::new();
        mock_current_platform_retriever
            .expect_get_current_platform()
            .once()
            .return_const(Platform::new("linux", "x86_64"));
        let mut mock_download = MockFileDownloaderTrait::new();
        mock_download
            .expect_download_file_to_directory()
            .with(
                eq("https://github.com/Solipath/Solipath/releases/download/latest-linux_x86_64/solipath"),
                eq(solipath_temp_dir.clone()),
            )
            .once()
            .returning(move |_, _| {
                fs::write(&fake_solipath_executable, "not a real executable").unwrap();
                Ok(PathBuf::new())
            });
        let solipath_cli = SolipathCli {
            file_downloader: Arc::new(mock_download),
            directory_finder: Arc::new(mock_directory_finder),
            current_platform_retriever: Arc::new(mock_current_platform_retriever),
        };
        solipath_cli.run_solipath_command(&["--update".to_string()]).await.unwrap();
    }

    #[tokio::test]
    async fn linux_downloads_solipath_fails_download_and_moves_solipath_back() {
        let solipath_temp_dir = tempdir().unwrap().into_path();
        let mut fake_solipath_executable = solipath_temp_dir.clone();
        fake_solipath_executable.push("solipath");
        fs::write(&fake_solipath_executable, "not a real executable").unwrap();
        let mut mock_directory_finder = MockSolipathDirectoryFinderTrait::new();
        mock_directory_finder
            .expect_get_base_solipath_directory()
            .once()
            .return_const(solipath_temp_dir.clone());
        let mut mock_current_platform_retriever = MockCurrentPlatformRetrieverTrait::new();
        mock_current_platform_retriever
            .expect_get_current_platform()
            .once()
            .return_const(Platform::new("linux", "x86_64"));
        let mut mock_download = MockFileDownloaderTrait::new();
        mock_download
            .expect_download_file_to_directory()
            .with(
                eq("https://github.com/Solipath/Solipath/releases/download/latest-linux_x86_64/solipath"),
                eq(solipath_temp_dir.clone()),
            )
            .once()
            .returning(move |_, _| Err(Error::msg("something went wrong")));
        let solipath_cli = SolipathCli {
            file_downloader: Arc::new(mock_download),
            directory_finder: Arc::new(mock_directory_finder),
            current_platform_retriever: Arc::new(mock_current_platform_retriever),
        };
        assert!(solipath_cli.run_solipath_command(&["--update".to_string()]).await.is_err());
        assert_eq!(
            "not a real executable".to_string(),
            String::from_utf8(fs::read(&fake_solipath_executable).unwrap()).unwrap()
        );
    }

    #[tokio::test]
    async fn windows_downloads_an_exe() {
        let solipath_temp_dir = tempdir().unwrap().into_path();
        let mut fake_solipath_executable = solipath_temp_dir.clone();
        fake_solipath_executable.push("solipath.exe");
        fs::write(&fake_solipath_executable, "not a real executable").unwrap();
        let mut mock_directory_finder = MockSolipathDirectoryFinderTrait::new();
        mock_directory_finder
            .expect_get_base_solipath_directory()
            .once()
            .return_const(solipath_temp_dir.clone());
        let mut mock_current_platform_retriever = MockCurrentPlatformRetrieverTrait::new();
        mock_current_platform_retriever
            .expect_get_current_platform()
            .once()
            .return_const(Platform::new("windows", "x86_64"));
        let mut mock_download = MockFileDownloaderTrait::new();
        mock_download
            .expect_download_file_to_directory()
            .with(
                eq("https://github.com/Solipath/Solipath/releases/download/latest-windows_x86_64/solipath.exe"),
                eq(solipath_temp_dir.clone()),
            )
            .once()
            .returning(move |_, _| {
                fs::write(&fake_solipath_executable, "not a real executable").unwrap();
                Ok(PathBuf::new())
            });
        let solipath_cli = SolipathCli {
            file_downloader: Arc::new(mock_download),
            directory_finder: Arc::new(mock_directory_finder),
            current_platform_retriever: Arc::new(mock_current_platform_retriever),
        };
        solipath_cli.run_solipath_command(&["--update".to_string()]).await.unwrap();
    }
}
