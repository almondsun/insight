use crate::fame::Precision;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub const FIXED_RECORD_BYTES: usize = 64;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CorpusStatus {
    Available,
    Private,
    Missing,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CorpusInputRecord {
    pub username: String,
    pub followers: Option<u64>,
    pub following: Option<u64>,
    pub precision: Option<Precision>,
    pub observed_at: String,
    pub status: CorpusStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodedCorpusRecord {
    normalized_username: String,
    index: u64,
    bytes: [u8; FIXED_RECORD_BYTES],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntheticCorpusRelease {
    schema_version: u32,
    release_id: String,
    source: String,
    records: Vec<EncodedCorpusRecord>,
    commitment_sha256: String,
}

impl EncodedCorpusRecord {
    pub fn normalized_username(&self) -> &str {
        &self.normalized_username
    }

    pub fn index(&self) -> u64 {
        self.index
    }

    pub fn bytes(&self) -> &[u8; FIXED_RECORD_BYTES] {
        &self.bytes
    }
}

impl SyntheticCorpusRelease {
    pub fn schema_version(&self) -> u32 {
        self.schema_version
    }

    pub fn release_id(&self) -> &str {
        &self.release_id
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn records(&self) -> &[EncodedCorpusRecord] {
        &self.records
    }

    pub fn commitment_sha256(&self) -> &str {
        &self.commitment_sha256
    }

    pub fn verify(&self) -> Result<(), String> {
        if self.schema_version != 1
            || self.release_id.trim().is_empty()
            || self.source.trim().is_empty()
        {
            return Err("Corpus release metadata is invalid".into());
        }
        for (expected_index, record) in self.records.iter().enumerate() {
            if record.index != expected_index as u64
                || !crate::identity::is_valid_username(&record.normalized_username)
                || record.normalized_username
                    != crate::identity::normalize(&record.normalized_username)
            {
                return Err("Corpus record ordering or identity is invalid".into());
            }
            let expected_digest = Sha256::digest(record.normalized_username.as_bytes());
            if record.bytes[32..64] != expected_digest[..] {
                return Err("Corpus username mapping is invalid".into());
            }
        }
        let expected = release_commitment(
            self.schema_version,
            &self.release_id,
            &self.source,
            &self.records,
        );
        if self.commitment_sha256 != expected {
            return Err("Corpus release commitment is invalid".into());
        }
        Ok(())
    }
}

pub fn build_synthetic_release(
    release_id: &str,
    source: &str,
    records: Vec<CorpusInputRecord>,
) -> Result<SyntheticCorpusRelease, String> {
    if release_id.trim().is_empty() || source.trim().is_empty() {
        return Err("Corpus release identity is required".into());
    }
    let mut normalized = BTreeMap::new();
    for record in records {
        let username = record.username.trim().to_lowercase();
        if !crate::identity::is_valid_username(&username) {
            return Err("Corpus contains an invalid username".into());
        }
        if normalized.insert(username, record).is_some() {
            return Err("Corpus contains a duplicate username".into());
        }
    }
    let mut encoded = Vec::with_capacity(normalized.len());
    for (index, (username, record)) in normalized.into_iter().enumerate() {
        let bytes = encode_record(&username, &record)?;
        encoded.push(EncodedCorpusRecord {
            normalized_username: username,
            index: index as u64,
            bytes,
        });
    }
    let commitment_sha256 = release_commitment(1, release_id, source, &encoded);
    let release = SyntheticCorpusRelease {
        schema_version: 1,
        release_id: release_id.into(),
        source: source.into(),
        records: encoded,
        commitment_sha256,
    };
    release.verify()?;
    Ok(release)
}

fn release_commitment(
    schema_version: u32,
    release_id: &str,
    source: &str,
    records: &[EncodedCorpusRecord],
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"insight-synthetic-corpus-release-v1\0");
    hasher.update(schema_version.to_be_bytes());
    hasher.update((release_id.len() as u64).to_be_bytes());
    hasher.update(release_id.as_bytes());
    hasher.update((source.len() as u64).to_be_bytes());
    hasher.update(source.as_bytes());
    hasher.update((records.len() as u64).to_be_bytes());
    for record in records {
        hasher.update(record.index.to_be_bytes());
        hasher.update(record.bytes);
    }
    format!("{:x}", hasher.finalize())
}

fn encode_record(
    normalized_username: &str,
    record: &CorpusInputRecord,
) -> Result<[u8; FIXED_RECORD_BYTES], String> {
    let (status, precision, followers, following) = match record.status {
        CorpusStatus::Available => (
            1_u8,
            match record
                .precision
                .ok_or("Available record requires precision")?
            {
                Precision::Exact => 1,
                Precision::Approximate => 2,
            },
            record
                .followers
                .ok_or("Available record requires followers")?,
            record
                .following
                .ok_or("Available record requires following")?,
        ),
        CorpusStatus::Private => (2, 0, 0, 0),
        CorpusStatus::Missing => (3, 0, 0, 0),
    };
    let observed_at = chrono::DateTime::parse_from_rfc3339(&record.observed_at)
        .map_err(|_| "Corpus observation time must be RFC 3339")?
        .timestamp();
    let username_digest = Sha256::digest(normalized_username.as_bytes());
    let mut bytes = [0_u8; FIXED_RECORD_BYTES];
    bytes[0] = status;
    bytes[1] = precision;
    bytes[8..16].copy_from_slice(&followers.to_be_bytes());
    bytes[16..24].copy_from_slice(&following.to_be_bytes());
    bytes[24..32].copy_from_slice(&observed_at.to_be_bytes());
    bytes[32..64].copy_from_slice(&username_digest);
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn available(username: &str) -> CorpusInputRecord {
        CorpusInputRecord {
            username: username.into(),
            followers: Some(100),
            following: Some(10),
            precision: Some(Precision::Exact),
            observed_at: "2026-01-01T00:00:00Z".into(),
            status: CorpusStatus::Available,
        }
    }

    #[test]
    fn release_is_fixed_width_and_deterministic() {
        let first = build_synthetic_release(
            "r1",
            "synthetic",
            vec![available("Bob"), available("alice")],
        )
        .unwrap();
        let second = build_synthetic_release(
            "r1",
            "synthetic",
            vec![available("alice"), available("Bob")],
        )
        .unwrap();
        assert_eq!(first, second);
        assert!(first
            .records()
            .iter()
            .all(|record| record.bytes().len() == 64));
        assert_eq!(first.records()[0].normalized_username(), "alice");
        assert!(first.verify().is_ok());
    }

    #[test]
    fn rejects_duplicates_and_incomplete_available_records() {
        assert!(build_synthetic_release(
            "r1",
            "synthetic",
            vec![available("Alice"), available("alice")],
        )
        .is_err());
        let mut incomplete = available("alice");
        incomplete.followers = None;
        assert!(build_synthetic_release("r1", "synthetic", vec![incomplete]).is_err());
    }
}
