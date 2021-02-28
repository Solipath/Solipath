use std::env::consts::ARCH;
use std::env::consts::OS;

#[cfg(test)]
use mockall::automock;

use crate::solipath_platform::platform::Platform;

#[cfg_attr(test, automock)]
pub trait CurrentPlatformRetrieverTrait {
    fn get_current_platform(&self) -> Platform;
}

pub struct CurrentPlatformRetriever;

impl CurrentPlatformRetriever {
    pub fn new() -> Self {
        Self {}
    }
}

impl CurrentPlatformRetrieverTrait for CurrentPlatformRetriever {
    fn get_current_platform(&self) -> Platform {
        Platform::new(OS, ARCH)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_get_current_platform() {
        assert_eq!(
            CurrentPlatformRetriever::new().get_current_platform(),
            Platform::new(OS, ARCH)
        );
    }
}
