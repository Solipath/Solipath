use solipath_lib::solipath_platform::platform::Platform;
use solipath_lib::solipath_platform::platform_filter::PlatformFilterTrait;

pub struct UnfilteredPlatform;

impl UnfilteredPlatform{
    pub fn new()-> Self {
        Self{}
    }
}

impl PlatformFilterTrait for UnfilteredPlatform{
    fn current_platform_is_match(&self, _: &[Platform])-> bool{
        true
    }
}