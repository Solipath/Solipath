#[cfg(test)]
use mockall::automock;

use crate::solipath_instructions::data::template::Template;

#[cfg_attr(test, automock)]
pub trait TemplateVariableReplacerTrait {
    fn replace_variables(&self, input: &str, template: &Template) -> String;
}

pub struct TemplateVariableReplacer;

impl TemplateVariableReplacer {
    pub fn new() -> Self {
        Self {}
    }
}

impl TemplateVariableReplacerTrait for TemplateVariableReplacer {
    fn replace_variables(&self, input: &str, template: &Template) -> String {
        let mut output = input.to_string();
        template.get_variables().iter().for_each(|(key, value)| {
            let string_to_replace = format!("${{{}}}", key);
            output = output.replace(&string_to_replace, &value);
        });
        output
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn returns_same_string_if_no_variables() {
        let variable_replacer = TemplateVariableReplacer::new();
        let template = serde_json::from_str::<Template>(r#"{"name": "something"}"#).unwrap();
        let input = r#"{"downloads": [{"url": "google.com", "destination_directory": "/something"}]}"#;
        let expected = r#"{"downloads": [{"url": "google.com", "destination_directory": "/something"}]}"#;
        assert_eq!(variable_replacer.replace_variables(input, &template), expected);
    }

    #[test]
    fn single_variable_replacement() {
        let variable_replacer = TemplateVariableReplacer::new();
        let template =
            serde_json::from_str::<Template>(r#"{"name": "something", "variables": {"key1": "the-link"}}"#).unwrap();
        let input = r#"{"downloads": [{"url": "${key1}.com", "destination_directory": "/something"}]}"#;
        let expected = r#"{"downloads": [{"url": "the-link.com", "destination_directory": "/something"}]}"#;
        assert_eq!(variable_replacer.replace_variables(input, &template), expected);
    }

    #[test]
    fn multiple_variable_replacement() {
        let variable_replacer = TemplateVariableReplacer::new();
        let template = serde_json::from_str::<Template>(
            r#"{"name": "something", "variables": {"key1": "the-link", "key2": "the/path"}}"#,
        )
        .unwrap();
        let input = r#"{"downloads": [{"url": "${key1}.com", "destination_directory": "/${key2}"}]}"#;
        let expected = r#"{"downloads": [{"url": "the-link.com", "destination_directory": "/the/path"}]}"#;
        assert_eq!(variable_replacer.replace_variables(input, &template), expected);
    }
}
