use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::Arc;
use crate::solipath_instructions::data::dependency::Dependency;
use crate::solipath_instructions::data::dependency_instructions::DependencyInstructions;
use crate::{
    solipath_directory::solipath_directory_finder::SolipathDirectoryFinderTrait,
    solipath_download::file_to_string_downloader::FileToStringDownloaderTrait,
};

#[cfg(test)]
use mockall::{automock, predicate::*};

#[cfg_attr(test, automock)]
#[async_trait]
pub trait DependencyInstructionsRetrieverTrait {
    async fn retrieve_dependency_instructions(&self, depend: &Dependency) -> DependencyInstructions;
}

pub struct DependencyInstructionsRetriever {
    base_dependency_url: String,
    file_downloader: Arc<dyn FileToStringDownloaderTrait + Sync + Send>,
    directory_finder: Arc<dyn SolipathDirectoryFinderTrait + Sync + Send>,
}

impl DependencyInstructionsRetriever {
    pub fn new(
        file_downloader: Arc<dyn FileToStringDownloaderTrait + Sync + Send>,
        directory_finder: Arc<dyn SolipathDirectoryFinderTrait + Sync + Send>,
    ) -> Self {
        Self {
            base_dependency_url: "https://raw.githubusercontent.com/Solipath/Solipath-Install-Instructions/main".to_string(),
            file_downloader,
            directory_finder,
        }
    }

    pub fn new_with_alternate_url(
        base_dependency_url: String,
        file_downloader: Arc<dyn FileToStringDownloaderTrait + Sync + Send>,
        directory_finder: Arc<dyn SolipathDirectoryFinderTrait + Sync + Send>,
    ) -> Self {
        Self {
            base_dependency_url,
            file_downloader,
            directory_finder,
        }
    }


    fn get_path_to_save_file(&self, dependency: &Dependency) -> PathBuf {
        let mut path_to_save_file = self.directory_finder.get_dependency_version_directory(&dependency);
        path_to_save_file.push("install_instructions.json");
        path_to_save_file
    }

    fn get_url(&self, dependency: &Dependency) -> String {
        format!(
            "{}/{}/{}/install_instructions.json",
            &self.base_dependency_url, dependency.name, dependency.version
        )
    }
}

#[async_trait]
impl DependencyInstructionsRetrieverTrait for DependencyInstructionsRetriever {
    async fn retrieve_dependency_instructions(&self, dependency: &Dependency) -> DependencyInstructions {
        let path_to_save_file = self.get_path_to_save_file(dependency);
        let url = self.get_url(dependency);
        let dependency_json_string = self
            .file_downloader
            .download_file_then_parse_to_string(&url, &path_to_save_file)
            .await;
        DependencyInstructions::new(
            dependency.clone(),
            serde_json::from_str(&dependency_json_string).expect(&format!("failed to serialize install instructions {}", &dependency_json_string)),
        )
    }
}

#[cfg(test)]
mod test {

    use std::path::PathBuf;

    use mockall::predicate::eq;

    use super::*;
    use crate::solipath_directory::solipath_directory_finder::MockSolipathDirectoryFinderTrait;
    use crate::solipath_download::file_to_string_downloader::MockFileToStringDownloaderTrait;
    use crate::solipath_instructions::data::install_instructions::InstallInstructions;

    #[tokio::test]
    async fn can_retrieve_dependencies_does_not_download_if_exists_already() {
        let input_dependency = Dependency::new("name", "version");
        let install_instructions: InstallInstructions = serde_json::from_str("{}").unwrap();
        let expected = DependencyInstructions::new(input_dependency.clone(), install_instructions.clone());

        let dependency_directory = PathBuf::new();
        let mut path_to_downloaded_file = dependency_directory.clone();
        path_to_downloaded_file.push("install_instructions.json");

        let mut mock_directory_finder = MockSolipathDirectoryFinderTrait::new();
        mock_directory_finder
            .expect_get_dependency_version_directory()
            .with(eq(input_dependency.clone()))
            .times(1)
            .return_const(dependency_directory);

        let mut mock_file_downloader = MockFileToStringDownloaderTrait::new();
        mock_file_downloader
            .expect_download_file_then_parse_to_string()
            .withf(move |url, path| {
                url == "https://raw.githubusercontent.com/Solipath/Solipath-Install-Instructions/main/name/version/install_instructions.json"
                    && path == path_to_downloaded_file.clone()
            })
            .times(1)
            .return_const("{}");

        let file_retriever =
            DependencyInstructionsRetriever::new(Arc::new(mock_file_downloader), Arc::new(mock_directory_finder));

        let actual = file_retriever.retrieve_dependency_instructions(&input_dependency).await;
        assert_eq!(actual, expected);
    }
}
