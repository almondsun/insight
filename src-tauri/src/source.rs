use crate::model::ParsedImport;
use std::path::Path;

pub trait RelationshipSourceAdapter {
    fn source_id(&self) -> &'static str;
    fn parse(&self, path: &Path) -> Result<ParsedImport, String>;
}

pub struct InstagramArchiveV1;

impl RelationshipSourceAdapter for InstagramArchiveV1 {
    fn source_id(&self) -> &'static str {
        "instagram-archive-v1"
    }

    fn parse(&self, path: &Path) -> Result<ParsedImport, String> {
        crate::parser::parse_path(path)
    }
}

pub fn parse(path: &Path) -> Result<ParsedImport, String> {
    let adapter = InstagramArchiveV1;
    let _source_id = adapter.source_id();
    adapter.parse(path)
}
