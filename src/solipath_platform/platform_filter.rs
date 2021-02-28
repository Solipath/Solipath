use std::sync::Arc;

#[cfg(test)]
use mockall::automock;

use crate::solipath_platform::current_platform_retriever::CurrentPlatformRetrieverTrait;
use crate::solipath_platform::platform::Platform;

#[cfg_attr(test, automock)]
pub trait PlatformFilterTrait {
    fn current_platform_is_match(&self, platform_filter: &[Platform]) -> bool;
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
        platform_list.iter().fold(false, |accumulator, platform| {
            accumulator || platform.is_superset_of(&current_platform)
        })
    }
}

impl PlatformFilterTrait for PlatformFilter {
    fn current_platform_is_match(&self, platform_list: &[Platform]) -> bool {
        platform_list.is_empty() || self.match_found_in_list(platform_list)
    }
}

#[cfg(test)]
mod test {
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
}

#[cfg(test)]
pub mod mock {
    use crate::solipath_platform::platform::Platform;
    use crate::solipath_platform::platform_filter::MockPlatformFilterTrait;

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
