use flate2::read::GzDecoder;
use sevenz_rust::decompress_file;
use std::fs;
use std::fs::create_dir_all;
use std::fs::File;
use std::io::BufReader;
use std::io::Cursor;
use std::path::Path;
use tar::Archive;
use zip::ZipArchive;

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait FileDecompressorTrait {
    fn decompress_file_to_directory(&self, source_file: &Path, target_directory: &Path);
}

pub struct FileDecompressor;

impl FileDecompressor {
    pub fn new() -> Self {
        Self {}
    }
}

impl FileDecompressorTrait for FileDecompressor {
    fn decompress_file_to_directory(&self, source_file: &Path, target_directory: &Path) {
        let file_name = source_file.file_name().unwrap().to_str().unwrap();
        create_dir_all(&target_directory).expect("failed to create parent directories");
        println!("starting to move {} to {:?}", file_name, target_directory);
        if file_name.ends_with(".zip") {
            unzip_to_destination(source_file, target_directory);
        } else if file_name.ends_with(".tar.gz") || file_name.ends_with(".tgz") {
            extract_tar_gz_to_destination(source_file, target_directory);
        } else if file_name.ends_with(".tar.xz") {
            extract_tar_xz_to_destination(source_file, target_directory);
        } else if file_name.ends_with(".7z") {
            extract_7z_to_destination(source_file, target_directory);
        } else {
            just_copy_file_to_destination(source_file, target_directory, file_name);
        }
        println!("finished moving {} to {:?}", file_name, target_directory);
    }
}

fn extract_tar_xz_to_destination(source_file: &Path, target_directory: &Path) {
    let tar_xz_file = File::open(source_file).expect("failed to open file");
    let mut tar_data = Vec::with_capacity(tar_xz_file.metadata().unwrap().len() as usize);
    let mut buffered_reader = BufReader::new(tar_xz_file);
    lzma_rs::xz_decompress(&mut buffered_reader, &mut tar_data).expect("failed to decompresss xz file");
    let mut tar_cursor = Cursor::new(tar_data);
    let mut archive = Archive::new(&mut tar_cursor);
    archive.unpack(&target_directory).expect("failed to extract tar file");
}

fn extract_7z_to_destination(source_file: &Path, target_directory: &Path) {
    decompress_file(source_file, target_directory).expect("failed to extract file");
}

fn unzip_to_destination(source_file: &Path, target_directory: &Path) {
    let zip_file = File::open(source_file).expect("failed to open file");
    let buffered_reader = BufReader::new(zip_file);
    ZipArchive::new(buffered_reader)
        .expect("failed to open zip file")
        .extract(target_directory)
        .expect("failed to extract file");
}

fn extract_tar_gz_to_destination(source_file: &Path, target_directory: &Path) {
    let tar_gz = File::open(source_file).expect("failed to open file");
    let buffered_reader = BufReader::new(tar_gz);
    let tar = GzDecoder::new(buffered_reader);
    let mut archive = Archive::new(tar);
    archive.unpack(target_directory).expect("failed to extract tar file");
}

fn just_copy_file_to_destination(source_file: &Path, target_directory: &Path, file_name: &str) {
    let mut target_file = target_directory.to_path_buf();
    target_file.push(file_name);
    fs::copy(source_file, target_file).expect("failed to copy file");
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn just_copies_file_to_destination_folder_if_not_zip_or_targz_file() {
        let temp_dir = tempdir().unwrap();
        let mut target_directory = temp_dir.path().to_path_buf();
        target_directory.push("additional_node");
        let mut expected_destination_file = target_directory.clone();
        expected_destination_file.push("simple_file.txt");
        let mut source_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        source_file.push("tests/resources/simple_file.txt");

        let file_decompressor = FileDecompressor::new();
        file_decompressor.decompress_file_to_directory(&source_file, &target_directory);

        let file_contents = fs::read_to_string(expected_destination_file.to_str().unwrap())
            .expect("something went wrong trying to read file");
        assert_eq!(file_contents, "this is a simple uncompressed file used for testing");
    }

    #[test]
    fn decompresses_zip_file_to_destination_directory() {
        let temp_dir = tempdir().unwrap();
        let target_directory = temp_dir.path().to_path_buf();
        let mut expected_destination_file = target_directory.clone();
        expected_destination_file.push("file_in_zip.txt");
        let mut source_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        source_file.push("tests/resources/zip_file.zip");

        let file_decompressor = FileDecompressor::new();
        file_decompressor.decompress_file_to_directory(&source_file, &target_directory);

        let file_contents = fs::read_to_string(expected_destination_file.to_str().unwrap())
            .expect("something went wrong trying to read file");
        assert_eq!(file_contents, "this file is inside a zip file\n");
    }

    #[test]
    fn decompresses_tar_gz_file_to_destination_directory() {
        let temp_dir = tempdir().unwrap();
        let target_directory = temp_dir.path().to_path_buf();
        let mut expected_destination_file = target_directory.clone();
        expected_destination_file.push("file_in_tar_gz.txt");
        let mut source_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        source_file.push("tests/resources/tar_gz_file.tar.gz");

        let file_decompressor = FileDecompressor::new();
        file_decompressor.decompress_file_to_directory(&source_file, &target_directory);

        let file_contents = fs::read_to_string(expected_destination_file.to_str().unwrap())
            .expect("something went wrong trying to read file");
        assert_eq!(file_contents, "this is a file inside a .tar.gz\n");
    }
    #[test]
    fn decompresses_tar_gz_file_to_destination_directory_with_tgz_extension() {
        let temp_dir = tempdir().unwrap();
        let target_directory = temp_dir.path().to_path_buf();
        let mut expected_destination_file = target_directory.clone();
        expected_destination_file.push("file_in_tar_gz.txt");
        let mut source_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        source_file.push("tests/resources/tar_gz_file.tgz");

        let file_decompressor = FileDecompressor::new();
        file_decompressor.decompress_file_to_directory(&source_file, &target_directory);

        let file_contents = fs::read_to_string(expected_destination_file.to_str().unwrap())
            .expect("something went wrong trying to read file");
        assert_eq!(file_contents, "this is a file inside a .tar.gz\n");
    }

    #[test]
    fn decompresses_7z_file_to_destination_directory_with_7z_extension() {
        let temp_dir = tempdir().unwrap();
        let target_directory = temp_dir.path().to_path_buf();
        let mut expected_destination_file = target_directory.clone();
        expected_destination_file.push("file_in_7z.txt");
        let mut source_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        source_file.push("tests/resources/7z_file.7z");

        let file_decompressor = FileDecompressor::new();
        file_decompressor.decompress_file_to_directory(&source_file, &target_directory);

        let file_contents = fs::read_to_string(expected_destination_file.to_str().unwrap())
            .expect("something went wrong trying to read file");
        assert_eq!(file_contents, "this is a file inside a .7z");
    }

    #[test]
    fn decompresses_tar_xz_file_to_destination_directory_with_tar_xz_extension() {
        let temp_dir = tempdir().unwrap();
        let target_directory = temp_dir.path().to_path_buf();
        let mut expected_destination_file = target_directory.clone();
        expected_destination_file.push("file_in_tar_xz.txt");
        let mut source_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        source_file.push("tests/resources/tar_xz_file.tar.xz");

        let file_decompressor = FileDecompressor::new();
        file_decompressor.decompress_file_to_directory(&source_file, &target_directory);

        let file_contents = fs::read_to_string(expected_destination_file.to_str().unwrap())
            .expect("something went wrong trying to read file");
        assert_eq!(file_contents, "this is a file inside a .tar.xz");
    }
}
