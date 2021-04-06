use std::path::Path;

use solipath_lib::solipath_download::file_decompressor::FileDecompressorTrait;

pub struct FakeDecompressor;

impl FakeDecompressor{
    pub fn new()->Self{
        Self{}
    }
}

impl FileDecompressorTrait for FakeDecompressor{
    fn decompress_file_to_directory(&self, _: &Path, _: &Path) {
    }
}