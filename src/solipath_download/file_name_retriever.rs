use reqwest::header::HeaderMap;
use reqwest::header::CONTENT_DISPOSITION;

pub fn get_file_name(url: &str, header: &HeaderMap) -> String {
    if let Some(file_name) = get_file_name_from_content_disposition(header) {
        file_name
    } else {
        get_string_after_last_forward_slash(url)
    }
}

fn get_file_name_from_content_disposition(header: &HeaderMap) -> Option<String> {
    if header.contains_key(CONTENT_DISPOSITION) {
        let content_disposition= header
        .get(CONTENT_DISPOSITION)
        .expect("header should have had a content disposition")
        .to_str()
        .expect("content disposition should be able to be converted to a string");
        mailparse::parse_content_disposition(content_disposition).params.get("filename").and_then(|file_name|{
            Some(file_name.to_owned())
        })
    } else {
        None
    }
}


fn get_string_after_last_forward_slash(url: &str) -> String {
    let index_of_forward_slash: usize = url.rfind('/').expect("could not find a forward slash in url");
    let (_, string_after_last_slash) = url.split_at(index_of_forward_slash + 1);
    if let Some(index_of_question_mark) = string_after_last_slash.rfind('?') {
        let (file_name, _) = string_after_last_slash.split_at(index_of_question_mark);
        file_name.to_string()
    } else {
        string_after_last_slash.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_file_name_from_response_no_content_disposition() {
        let url = "https://download.com/something.json";
        let header = HeaderMap::new();
        assert_eq!(get_file_name(url, &header), "something.json");
    }

    #[test]
    fn get_file_name_from_response_no_content_disposition_but_with_query_parameter() {
        let url = "https://download.com/android-studio-2023.3.1.18-linux.tar.gz?cms_redirect=yes&mh=0E&mip=67.149.128.201&mm=28&mn=sn-fvf-quf6&ms=nvh&mt=1717283768&mv=m&mvi=3&pcm2cms=yes&pl=21&shardbypass=sd";
        let header = HeaderMap::new();
        assert_eq!(get_file_name(url, &header), "android-studio-2023.3.1.18-linux.tar.gz");
    }

    #[test]
    fn get_file_name_from_response_simple_content_disposition_with_file_name() {
        let url = "https://download.com/";
        let mut header = HeaderMap::new();
        header.append(
            CONTENT_DISPOSITION,
            "attachment; filename=\"filename.jpg\"".parse().unwrap(),
        );
        assert_eq!(get_file_name(url, &header), "filename.jpg");
    }

    #[test]
    fn get_file_name_from_response_simple_content_disposition_with_utf8_file_name() {
        let url = "https://download.com/";
        let mut header = HeaderMap::new();
        header.append(
            CONTENT_DISPOSITION,
            "attachment; filename*=UTF-8''filename2.jpg".parse().unwrap(),
        );
        assert_eq!(get_file_name(url, &header), "filename2.jpg");
    }

    #[test]
    fn get_file_name_from_response_simple_content_disposition_with_only_attachment_should_use_url() {
        let url = "https://download.com/something2.jpg";
        let mut header = HeaderMap::new();
        header.append(CONTENT_DISPOSITION, "attachment".parse().unwrap());
        assert_eq!(get_file_name(url, &header), "something2.jpg");
    }

    #[test]
    fn get_string_after_last_forward_slash_should_not_include_leading_slash() {
        let url = "https://download.com/something.json";
        assert_eq!(get_string_after_last_forward_slash(url), "something.json");
    }
}
