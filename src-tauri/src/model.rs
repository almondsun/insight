use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub type People = BTreeMap<String, Person>;
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub id: i64,
    pub label: String,
    pub username: Option<String>,
    pub snapshot_count: i64,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub id: i64,
    pub account_id: i64,
    pub imported_at: String,
    pub source_name: String,
    pub followers: usize,
    pub following: usize,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    pub followers: usize,
    pub following: usize,
    pub mutuals: usize,
    pub not_following_back: usize,
    pub followers_not_followed_back: usize,
    pub new_followers: usize,
    pub lost_followers: usize,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Relationship {
    pub username: String,
    pub profile_url: Option<String>,
    pub kind: String,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Change {
    pub username: String,
    pub profile_url: Option<String>,
    pub category: String,
    pub direction: String,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportPreview {
    pub token: String,
    pub source_name: String,
    pub detected_username: Option<String>,
    pub followers: usize,
    pub following: usize,
    pub warnings: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Person {
    pub username: String,
    pub profile_url: Option<String>,
    pub timestamp: Option<i64>,
}
#[derive(Debug, Clone)]
pub struct ParsedImport {
    pub source_name: String,
    pub detected_username: Option<String>,
    pub followers: People,
    pub following: People,
    pub warnings: Vec<String>,
    pub hash: String,
}
