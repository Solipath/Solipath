use glob::glob;
use std::path::Path;
use solipath_lib::solipath_instructions::data::dependency_instructions::DependencyInstructions;
use solipath_lib::solipath_instructions::data::install_instructions::InstallInstructions;
use solipath_lib::solipath_dependency_metadata::dependency::Dependency;
use std::fs::read_to_string;
pub struct InstallFileLooper;


impl InstallFileLooper {
    pub fn new()-> Self{
        Self{}
    }

    pub fn retrieve_all_dependency_instructions(&self, path: &Path)-> Vec<DependencyInstructions> {
        let full_path = format!("{}/{}", path.to_str().unwrap(), "**/install_instructions.json");
        glob(&full_path).unwrap().into_iter()
        .filter(|path| path.is_ok())
        .map(|path| {
            let matched_path = path.expect("one of the paths are not valid!");
            println!("found {:?}", matched_path);
            let error_string = format!("failed to parse the file at path: {}", matched_path.to_str().unwrap());
            let instructions: InstallInstructions = serde_json::from_str(&read_to_string(&matched_path).unwrap()).expect(&error_string);
            let version_folder = matched_path.parent().expect(&error_string);
            let version = version_folder.file_name().expect(&error_string).to_str().expect(&error_string);
            let dependency_name = version_folder.parent().expect(&error_string).file_name().expect(&error_string).to_str().expect(&error_string);
            DependencyInstructions::new(Dependency::new(dependency_name, version), instructions)
        }).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs::create_dir_all;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn can_get_an_empty_list() {
        let temp_dir = tempdir().unwrap().into_path();
        let install_file_looper = InstallFileLooper::new();
        let dependency_instructions = install_file_looper.retrieve_all_dependency_instructions(&temp_dir);
        assert_eq!(dependency_instructions, Vec::new());
    }


    #[test]
    fn can_get_a_single_dependency_instruction() {
        let temp_dir = tempdir().unwrap().into_path();
        create_empty_json_at_path(&temp_dir, "name1/version1/install_instructions.json");
        let install_file_looper = InstallFileLooper::new();
        let dependency_instructions = install_file_looper.retrieve_all_dependency_instructions(&temp_dir);
        assert_eq!(dependency_instructions, vec!(
            DependencyInstructions::new(Dependency::new("name1", "version1"), serde_json::from_str("{}").unwrap())
        ));
    }

    #[test]
    fn can_get_multiple_dependency_instructions_while_filtering_out_non_matches() {
        let temp_dir = tempdir().unwrap().into_path();
        create_empty_json_at_path(&temp_dir, "name1/version1/install_instructions.json");
        create_empty_json_at_path(&temp_dir, "name2/version2/install_instructions.json");
        create_empty_json_at_path(&temp_dir, "name1/template/template1.json");
        let install_file_looper = InstallFileLooper::new();
        let dependency_instructions = install_file_looper.retrieve_all_dependency_instructions(&temp_dir);
        assert_eq!(dependency_instructions, vec!(
            DependencyInstructions::new(Dependency::new("name1", "version1"), serde_json::from_str("{}").unwrap()),
            DependencyInstructions::new(Dependency::new("name2", "version2"), serde_json::from_str("{}").unwrap())
        ));
    }

    fn create_empty_json_at_path(base_path: &Path, rest_of_path: &str){
        let mut file = base_path.to_path_buf();
        file.push(rest_of_path);
        create_dir_all(file.parent().unwrap()).expect("failed to create directory");
        File::create(file).unwrap().write(b"{}").expect("failed to write to file");
    }
}