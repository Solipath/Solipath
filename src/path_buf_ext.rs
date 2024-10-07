use std::path::PathBuf;

pub trait PathBufExt {
    fn clone_push(&self, path: &str)-> PathBuf;
}

impl PathBufExt for PathBuf {
    fn clone_push(&self, path: &str)-> PathBuf {
        let mut new_path = self.clone();
        new_path.push(path);
        new_path
    }
}

#[cfg(test)]
mod test {
    use std::{path::PathBuf, str::FromStr};
    use super::*;
    #[test]
    fn can_clone_and_push_directory(){
        let original_path = PathBuf::from_str(".").unwrap();
        assert_eq!(PathBuf::from_str("./next_dir").unwrap(), original_path.clone_push("next_dir"))
    }
}