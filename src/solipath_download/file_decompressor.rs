use dmg::Attach;
use flate2::read::GzDecoder;
use zip::ZipArchive;
use std::fs::read_dir;
use std::io::Read;
use std::path::PathBuf;
use sevenz_rust::decompress_file;
use std::fs;
use std::fs::create_dir_all;
use std::fs::File;
use std::io::BufReader;
use std::io::Cursor;
use std::path::Path;
use tar::Archive;
use bzip2_rs::decoder::DecoderReader;

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
        } else if file_name.ends_with(".tar.bz2") {
            extract_tar_bz2_to_destination(source_file, target_directory);
        } else if file_name.ends_with(".7z") {
            extract_7z_to_destination(source_file, target_directory);
        } else if file_name.ends_with(".dmg"){
            extract_dmg_to_destination(source_file, target_directory);
        } else {
            just_copy_file_to_destination(source_file, target_directory, file_name);
        }
        println!("finished moving {} to {:?}", file_name, target_directory);
    }
}

fn recurse(path: impl AsRef<Path>) -> Vec<PathBuf> {
    let Ok(entries) = read_dir(path) else { return vec![]};
    entries.flatten().flat_map(|entry| {
        let Ok(meta) = entry.metadata() else {return vec![]};
        if meta.is_dir() {return recurse(entry.path());}
        if meta.is_symlink() {return vec![entry.path()];}
        if meta.is_file() {return vec![entry.path()];}
        vec![]
    }).collect()
}

fn extract_dmg_to_destination(source_file: &Path, target_directory: &Path) {
    let attached_dmg = Attach::new(source_file).mount_temp().hidden().force_readonly().with().expect("error attaching dmg");
    let attached_path = attached_dmg.mount_point.clone();
    recurse(&attached_path).iter().for_each(|source_path| {
        let relative_path = source_path.strip_prefix(&attached_path).expect("couldn't get relative path for dmg");
        let mut output_file_path = target_directory.to_path_buf();
        output_file_path.push(&relative_path);
        fs::create_dir_all(&output_file_path.parent().expect("failed to get parent dir")).expect("failed to create parent directory");
        if source_path.is_symlink() {
            std::os::unix::fs::symlink(fs::read_link(source_path).expect("failed to read symlink"), output_file_path).expect("failed to create symlink");
        } else if source_path.is_file() {
            fs::copy(source_path, output_file_path).expect("failed to copy file for dmg");
        }
    })
}

fn extract_tar_bz2_to_destination(source_file: &Path, target_directory: &Path) {
    let tar_bz2_file = File::open(source_file).expect("failed to open file");
    let mut decoder_reader = DecoderReader::new(&tar_bz2_file);
    let mut tar_data = Vec::new(); 
    decoder_reader.read_to_end(&mut tar_data).expect("failed to decompress bz2 file");
    let mut tar_cursor = Cursor::new(tar_data);
    let mut archive = Archive::new(&mut tar_cursor);
    archive.unpack(&target_directory).expect("failed to extract tar file");
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
    use std::fs::{self};
    use std::path::PathBuf;
    use std::str::FromStr;
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
    fn decompresses_zip_file_to_destination_directory_nested_folder() {
        let temp_dir = tempdir().unwrap();
        let target_directory = temp_dir.path().to_path_buf();
        let mut expected_destination_file = target_directory.clone();
        expected_destination_file.push("folder 1/file_in_zip.txt");
        let mut source_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        source_file.push("tests/resources/zip_file_nested_folder.zip");

        let file_decompressor = FileDecompressor::new();
        file_decompressor.decompress_file_to_directory(&source_file, &target_directory);

        let file_contents = fs::read_to_string(expected_destination_file.to_str().unwrap())
            .expect("something went wrong trying to read file");
        assert_eq!(file_contents, "this file is nested inside a zip file\n");
    }

    #[cfg(not(target_os="windows"))]
    #[test]
    fn decompresses_zip_file_to_destination_directory_and_symlinks_still_work() {
        let temp_dir = tempdir().unwrap();
        let target_directory = temp_dir.path().to_path_buf();
        let mut expected_destination_file = target_directory.clone();
        expected_destination_file.push("mySymlink");
        let mut source_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        source_file.push("tests/resources/compressed_symlink.zip");

        let file_decompressor = FileDecompressor::new();
        file_decompressor.decompress_file_to_directory(&source_file, &target_directory);
        assert!(expected_destination_file.is_symlink());
        assert_eq!(PathBuf::from_str("./simple_file.txt").unwrap(), 
        fs::read_link(expected_destination_file).unwrap());
    }

    #[test]
    fn decompresses_tar_bz2_file_to_destination_directory() {
        let temp_dir = tempdir().unwrap();
        let target_directory = temp_dir.path().to_path_buf();
        let mut expected_destination_file = target_directory.clone();
        expected_destination_file.push("tar_bz2_file.txt");
        let mut source_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        source_file.push("tests/resources/tar_bz2_file.tar.bz2");

        let file_decompressor = FileDecompressor::new();
        file_decompressor.decompress_file_to_directory(&source_file, &target_directory);

        let file_contents = fs::read_to_string(expected_destination_file.to_str().unwrap())
            .expect("something went wrong trying to read file");
        assert_eq!(file_contents, "tar bz2 file");
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

    #[cfg(target_os="macos")]
    #[test]
    fn decompresses_dmg_file_to_destination_directory_with_dmg_extension() {
        let temp_dir = tempdir().unwrap();
        let target_directory = temp_dir.path().to_path_buf();
        let mut expected_destination_file = target_directory.clone();
        expected_destination_file.push("nestedFolder/testdmg.txt");
        let mut expected_symlink = target_directory.clone();
        expected_symlink.push("symlinkToTestdmg");

        let mut source_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        source_file.push("tests/resources/testdmg.dmg");

        let file_decompressor = FileDecompressor::new();
        file_decompressor.decompress_file_to_directory(&source_file, &target_directory);
        let file_contents = fs::read_to_string(expected_destination_file.to_str().unwrap())
        .expect("something went wrong trying to read file");
        assert_eq!("this is a dmg file\n", file_contents);
        assert!(expected_symlink.is_symlink());
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
