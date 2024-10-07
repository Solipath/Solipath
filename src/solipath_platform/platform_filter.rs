use std::sync::Arc;

use async_trait::async_trait;
#[cfg(test)]
use mockall::automock;

use crate::solipath_platform::current_platform_retriever::CurrentPlatformRetrieverTrait;
use crate::solipath_platform::platform::Platform;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait PlatformFilterTrait: Sync + Send {
    fn current_platform_is_match(&self, platform_filter: &[Platform]) -> bool;
}

pub fn filter_list<T>(platform_filter: &Arc<dyn PlatformFilterTrait>, list: &Vec<T>) -> Vec<T>
where
    T: HasPlatformFilter + Clone,
{
    list.into_iter()
        .filter(|item| platform_filter.current_platform_is_match(item.get_platform_filters()))
        .map(|item| item.clone())
        .collect()
}

pub struct PlatformFilter {
    current_platform_retriever: Arc<dyn CurrentPlatformRetrieverTrait + Send + Sync>,
}

impl PlatformFilter {
    pub fn new(current_platform_retriever: Arc<dyn CurrentPlatformRetrieverTrait + Send + Sync>) -> Self {
        Self {
            current_platform_retriever,
        }
    }

    fn match_found_in_list(&self, platform_list: &[Platform]) -> bool {
        let current_platform = self.current_platform_retriever.get_current_platform();
        platform_list
            .iter()
            .any(|platform| platform.is_superset_of(&current_platform))
    }
}

impl PlatformFilterTrait for PlatformFilter {
    fn current_platform_is_match(&self, platform_list: &[Platform]) -> bool {
        platform_list.is_empty() || self.match_found_in_list(platform_list)
    }
}

pub trait HasPlatformFilter {
    fn get_platform_filters(&self) -> &[Platform];
}

#[cfg(test)]
mod test {
    use mock::FakeCurrentPlatformRetriever;

    use super::*;
    use crate::solipath_platform::current_platform_retriever::MockCurrentPlatformRetrieverTrait;

    #[test]
    fn empty_platform_filter_list_returns_true() {
        let current_platform_retriever = MockCurrentPlatformRetrieverTrait::new();
        let platform_filter = PlatformFilter::new(Arc::new(current_platform_retriever));

        assert_eq!(platform_filter.current_platform_is_match(&Vec::new()), true);
    }

    #[test]
    fn one_item_that_does_not_match_current_operating_system_returns_false() {
        let mut current_platform_retriever = MockCurrentPlatformRetrieverTrait::new();
        current_platform_retriever
            .expect_get_current_platform()
            .times(1)
            .return_const(Platform::new("linux", "x86_64"));
        let platform_filter = PlatformFilter::new(Arc::new(current_platform_retriever));
        let platform_list = vec![Platform::new("windows", "x86_64")];

        assert_eq!(platform_filter.current_platform_is_match(&platform_list), false);
    }

    #[test]
    fn two_items_one_match_operating_system_returns_true() {
        let mut current_platform_retriever = MockCurrentPlatformRetrieverTrait::new();
        current_platform_retriever
            .expect_get_current_platform()
            .times(1)
            .return_const(Platform::new("linux", "x86_64"));
        let platform_filter = PlatformFilter::new(Arc::new(current_platform_retriever));
        let platform_list = vec![Platform::new("windows", "x86_64"), Platform::new("linux", "x86_64")];
        assert_eq!(platform_filter.current_platform_is_match(&platform_list), true);
    }

    #[derive(Debug, PartialEq, Eq, Clone)]
    struct TestPlatformList {
        name: String,
        platform_filter: Vec<Platform>,
    }

    impl HasPlatformFilter for TestPlatformList {
        fn get_platform_filters(&self) -> &[Platform] {
            &self.platform_filter
        }
    }

    #[test]
    fn can_filter_has_platform_filter_list() {
        let platform_retriever = FakeCurrentPlatformRetriever {
            platform: Platform::new("Matching OS", "Matching Arch"),
        };
        let platform_filter: Arc<dyn PlatformFilterTrait> = Arc::new(PlatformFilter::new(Arc::new(platform_retriever)));
        let input = vec![
            TestPlatformList {
                name: "included nothing filtered".to_string(),
                platform_filter: Vec::new(),
            },
            TestPlatformList {
                name: "included no arch".to_string(),
                platform_filter: vec![Platform {
                    os: "Matching OS".to_string(),
                    arch: None,
                }],
            },
            TestPlatformList {
                name: "included exact match".to_string(),
                platform_filter: vec![Platform::new("Matching OS", "Matching Arch")],
            },
            TestPlatformList {
                name: "included one match".to_string(),
                platform_filter: vec![
                    Platform::new("Non Matching OS", "Non Match Arch"),
                    Platform::new("Matching OS", "Matching Arch"),
                ],
            },
            TestPlatformList {
                name: "not included".to_string(),
                platform_filter: vec![Platform::new("Non Matching OS", "Non Match Arch")],
            },
        ];

        assert_eq!(
            vec![
                "included nothing filtered".to_string(),
                "included no arch".to_string(),
                "included exact match".to_string(),
                "included one match".to_string()
            ],
            filter_list(&platform_filter, &input)
                .iter()
                .map(|output| output.name.to_string())
                .collect::<Vec<String>>()
        );
    }
}

#[cfg(test)]
pub mod mock {
    use crate::solipath_platform::current_platform_retriever::CurrentPlatformRetrieverTrait;
    use crate::solipath_platform::platform::Platform;
    use crate::solipath_platform::platform_filter::MockPlatformFilterTrait;

    pub struct FakeCurrentPlatformRetriever {
        pub platform: Platform,
    }

    impl CurrentPlatformRetrieverTrait for FakeCurrentPlatformRetriever {
        fn get_current_platform(&self) -> Platform {
            self.platform.clone()
        }
    }

    pub fn verify_platform_filter(
        platform_filter: &mut MockPlatformFilterTrait,
        platform_list: Vec<Platform>,
        return_value: bool,
        times_called: usize,
    ) {
        platform_filter
            .expect_current_platform_is_match()
            .withf(move |platform| platform == platform_list)
            .times(times_called)
            .return_const(return_value);
    }
}
