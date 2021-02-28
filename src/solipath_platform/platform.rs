use serde::Deserialize;
use std::option::Option;

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Platform {
    os: String,
    #[serde(default = "default_architecture")]
    arch: Option<String>,
}

impl Platform {
    pub fn new(operating_system: &str, architecture: &str) -> Self {
        Self {
            os: operating_system.to_string(),
            arch: Some(architecture.to_string()),
        }
    }
    pub fn is_superset_of(&self, other: &Platform) -> bool {
        (self.arch.is_none() && self.os == other.os) || self == other
    }
}

fn default_architecture() -> Option<String> {
    None
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn is_superset_returns_true_for_exact_match() {
        let platform = Platform::new("windows", "x86_64");
        assert_eq!(platform.is_superset_of(&Platform::new("windows", "x86_64")), true);
    }

    #[test]
    fn is_superset_returns_false_for_incorrect_match() {
        let platform = Platform::new("linux", "x64");
        assert_eq!(platform.is_superset_of(&Platform::new("windows", "x86_64")), false);
    }

    #[test]
    fn is_superset_returns_true_when_operating_system_matches_and_architecture_is_none_on_left_side() {
        let platform = Platform {
            os: "windows".to_string(),
            arch: None,
        };
        assert_eq!(platform.is_superset_of(&Platform::new("windows", "x86_64")), true);
    }
}
