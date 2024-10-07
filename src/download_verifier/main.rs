mod install_file_looper;
mod download_checker;
mod fake_decompressor;

use install_file_looper::InstallFileLooper;
use download_checker::DownloadChecker;
use solipath_lib::async_loop::run_async;
use solipath_lib::solipath_directory::moveable_home_directory_finder::MoveableHomeDirectoryFinder;
use solipath_lib::solipath_download::dependency_downloader::DependencyDownloaderTrait;
use solipath_lib::solipath_instructions::data::dependency_instructions::VecDependencyInstructions;
use solipath_lib::solipath_template::template_retriever::TemplateRetrieverTrait;
use std::path::PathBuf;
use std::sync::Arc;
use fake_decompressor::FakeDecompressor;
use solipath_lib::solipath_download::dependency_downloader::DependencyDownloader;
use solipath_lib::solipath_template::template_retriever::TemplateRetriever;
use solipath_lib::solipath_template::template_variable_replacer::TemplateVariableReplacer;
use solipath_lib::solipath_download::file_to_string_downloader::FileToStringDownloader;
use solipath_lib::solipath_download::conditional_file_downloader::ConditionalFileDownloader;



#[tokio::main]
async fn main() {
    let starting_path = PathBuf::from(".");
    run_for_path(starting_path).await;
}

async fn run_for_path(starting_path: PathBuf){
    let install_file_looper = InstallFileLooper::new();
    let download_checker = Arc::new(DownloadChecker::new());
    let moveable_home_directory_finder = Arc::new(MoveableHomeDirectoryFinder::new(starting_path.clone()));
    let file_decompressor = Arc::new(FakeDecompressor::new());
    let conditional_file_downloader = Arc::new(ConditionalFileDownloader::new(download_checker.clone(), file_decompressor));
    let template_variable_replacer = Arc::new(TemplateVariableReplacer::new());
    let file_to_string_downloader = Arc::new(FileToStringDownloader::new(conditional_file_downloader.clone()));
    let template_retriever = Arc::new(TemplateRetriever::new(file_to_string_downloader.clone(), moveable_home_directory_finder.clone(), template_variable_replacer.clone()));
    let dependency_downloader = Arc::new(DependencyDownloader::new(moveable_home_directory_finder.clone(), conditional_file_downloader.clone()));
    let mut dependency_instructions = install_file_looper.retrieve_all_dependency_instructions(&starting_path);
    let mut template_instructions = run_async(&dependency_instructions.get_templates(), |(dependency, template)| {
        template_retriever.retrieve_instructions_from_template(dependency, template)
    }).await;
    dependency_instructions.append(&mut template_instructions);
    run_async(&dependency_instructions.get_downloads(), |(dependency, download_instruction)|{
        dependency_downloader.download_dependency(dependency, download_instruction)
    }).await;
    println!("finished running!");
}

#[cfg(test)]
mod tests{
    use super::*;
    #[tokio::test]
    async fn quick_check_success_case(){
        let mut source_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        source_file.push("tests/resources/test_solipath_base_dir");
        run_for_path(source_file).await;
    }

    #[tokio::test]
    #[should_panic(expected = "url https://github.com/AdoptOpenJDK/openjdk11-binaries/releases/download/jdk-11.0.10%2B9/OpenJDK11U-jdk_x64_linux_hotspot_11.0.10_9.tar.gz.notreal failed to return")]
    async fn quick_check_fail_case(){
        let mut source_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        source_file.push("tests/resources/test_solipath_base_dir_with_bad_java_path");
        run_for_path(source_file).await;
    }
}