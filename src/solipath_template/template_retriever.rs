#[cfg(test)]
use mockall::automock;

use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::Arc;

use crate::solipath_instructions::data::dependency::Dependency;
use crate::solipath_directory::solipath_directory_finder::SolipathDirectoryFinderTrait;
use crate::solipath_download::file_to_string_downloader::FileToStringDownloaderTrait;
use crate::solipath_instructions::data::dependency_instructions::DependencyInstructions;
use crate::solipath_instructions::data::install_instructions::InstallInstructions;
use crate::solipath_instructions::data::template::Template;
use crate::solipath_template::template_variable_replacer::TemplateVariableReplacerTrait;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait TemplateRetrieverTrait {
    async fn retrieve_instructions_from_template(
        &self,
        dependency: &Dependency,
        template: &Template,
    ) -> DependencyInstructions;
}

pub struct TemplateRetriever {
    base_dependency_url: String,
    file_downloader: Arc<dyn FileToStringDownloaderTrait + Sync + Send>,
    directory_finder: Arc<dyn SolipathDirectoryFinderTrait + Sync + Send>,
    template_variable_replacer: Arc<dyn TemplateVariableReplacerTrait + Sync + Send>,
}

impl TemplateRetriever {
    pub fn new(
        file_downloader: Arc<dyn FileToStringDownloaderTrait + Sync + Send>,
        directory_finder: Arc<dyn SolipathDirectoryFinderTrait + Sync + Send>,
        template_variable_replacer: Arc<dyn TemplateVariableReplacerTrait + Sync + Send>,
    ) -> Self {
        Self {
            base_dependency_url: "https://raw.githubusercontent.com/Solipath/Solipath-Install-Instructions/main".to_string(),
            file_downloader,
            directory_finder,
            template_variable_replacer,
        }
    }
    pub fn new_with_alternate_url(
        base_dependency_url: String,
        file_downloader: Arc<dyn FileToStringDownloaderTrait + Sync + Send>,
        directory_finder: Arc<dyn SolipathDirectoryFinderTrait + Sync + Send>,
        template_variable_replacer: Arc<dyn TemplateVariableReplacerTrait + Sync + Send>,
    ) -> Self {
        Self {
            base_dependency_url,
            file_downloader,
            directory_finder,
            template_variable_replacer,
        }
    }

    fn get_path_to_save_file(&self, dependency: &Dependency, template: &Template) -> PathBuf {
        let mut path_to_save_file = self.directory_finder.get_dependency_template_directory(&dependency);
        path_to_save_file.push(format!("{}.json", template.get_name()));
        path_to_save_file
    }

    fn get_url(&self, dependency: &Dependency, template: &Template) -> String {
        format!(
            "{}/{}/templates/{}.json",
            &self.base_dependency_url,
            dependency.name,
            template.get_name()
        )
    }
}

#[async_trait]
impl TemplateRetrieverTrait for TemplateRetriever {
    async fn retrieve_instructions_from_template(
        &self,
        dependency: &Dependency,
        template: &Template,
    ) -> DependencyInstructions {
        let url = self.get_url(&dependency, &template);
        let output_path = self.get_path_to_save_file(dependency, template);
        let template_content = self
            .file_downloader
            .download_file_then_parse_to_string(&url, &output_path)
            .await;
        let replaced_template_content = self
            .template_variable_replacer
            .replace_variables(&template_content, &template);
        DependencyInstructions::new(
            dependency.clone(),
            serde_json::from_str::<InstallInstructions>(&replaced_template_content)
                .expect(&format!("failed to parse template {}", &replaced_template_content)),
        )
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use mockall::predicate::eq;
    use std::path::PathBuf;

    use crate::solipath_directory::solipath_directory_finder::MockSolipathDirectoryFinderTrait;
    use crate::solipath_download::file_to_string_downloader::MockFileToStringDownloaderTrait;
    use crate::solipath_template::template_variable_replacer::TemplateVariableReplacer;

    #[tokio::test]
    async fn retrieve_instructions_downloads_data_sends_data_to_template_retriever_and_builds_dependency_instructions()
    {
        let dependency = Dependency::new("java", "11");
        let template = serde_json::from_str::<Template>(
            r#"{"name": "template1", "variables": {"key1": "value1", "key2": "value2"}}"#,
        )
        .unwrap();
        let dependency_directory = PathBuf::from("/something");

        let mut mock_directory_finder = MockSolipathDirectoryFinderTrait::new();
        mock_directory_finder
            .expect_get_dependency_template_directory()
            .with(eq(dependency.clone()))
            .times(1)
            .return_const(dependency_directory);

        let mut mock_file_downloader = MockFileToStringDownloaderTrait::new();
        mock_file_downloader
            .expect_download_file_then_parse_to_string()
            .withf(move |url, path| {
                url == "https://raw.githubusercontent.com/Solipath/Solipath-Install-Instructions/main/java/templates/template1.json"
                && path == PathBuf::from("/something/template1.json")
            })
            .times(1)
            .return_const(r#"{"downloads": [{"url": "${key1}.com", "destination_directory": "/${key2}"}]}"#);

        let template_variable_replacer = TemplateVariableReplacer::new();

        let template_retriever = TemplateRetriever::new(
            Arc::new(mock_file_downloader),
            Arc::new(mock_directory_finder),
            Arc::new(template_variable_replacer),
        );
        let instructions = template_retriever
            .retrieve_instructions_from_template(&dependency, &template)
            .await;
        let expected = DependencyInstructions::new(
            dependency,
            serde_json::from_str::<InstallInstructions>(
                r#"{"downloads": [{"url": "value1.com", "destination_directory": "/value2"}]}"#,
            )
            .unwrap(),
        );
        assert_eq!(instructions, expected);
    }
}
