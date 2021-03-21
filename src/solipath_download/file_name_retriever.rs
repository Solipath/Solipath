use reqwest::header::HeaderMap;
use reqwest::header::HeaderValue;
use reqwest::header::CONTENT_DISPOSITION;
use hyperx::header::ContentDisposition;
use hyperx::header::DispositionParam;
use hyperx::header::Header;
use hyperx::header::Raw;


pub fn get_file_name(url: &str, header: &HeaderMap) -> String {
    match header.get(CONTENT_DISPOSITION) {
        Some(content_disposition) => get_file_name_from_content_disposition(content_disposition, url),
        None => get_string_after_last_forward_slash(url),
    }
}

fn get_file_name_from_content_disposition(header: &HeaderValue, url: &str) -> String {
    let content_string = header.to_str().expect("failed to convert to string");
    let content_disposition = ContentDisposition::parse_header::<Raw>(&content_string.into()).unwrap();
    let filename_param: Option<&DispositionParam> = content_disposition
        .parameters
        .iter()
        .filter(|param| is_filename(param))
        .next();
    if let Some(DispositionParam::Filename(_, _, file_bytes)) = filename_param {
        std::str::from_utf8(file_bytes).unwrap().to_string()
    } else {
        get_string_after_last_forward_slash(url)
    }
}

fn is_filename(param: &DispositionParam) -> bool {
    match param {
        DispositionParam::Filename(_, _, _) => true,
        _ => false,
    }
}

fn get_string_after_last_forward_slash(url: &str) -> String {
    let index_of_forward_slash: usize = url.rfind('/').expect("could not find a forward slash in url");
    let (_, string_after_last_slash) = url.split_at(index_of_forward_slash + 1);
    string_after_last_slash.to_string()
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn get_file_name_from_response_no_content_disposition() {
        let url = "https://download.com/something.json";
        let header = HeaderMap::new();
        assert_eq!(get_file_name(url, &header), "something.json");
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