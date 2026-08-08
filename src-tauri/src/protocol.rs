use serde::{Deserialize, Serialize};

pub const PROTOCOL_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CompatibilityPolicy {
    pub policy_id: String,
    pub minimum_secure_client: String,
    pub allowed_client_ranges: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EpochPolicy {
    pub epoch: u64,
    pub activates_at_unix_ms: i64,
    pub expires_at_unix_ms: i64,
    pub maximum_clock_skew_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolManifest {
    pub schema_version: u32,
    pub profile_id: String,
    pub compatibility_policy_id: String,
    pub corpus_release_id: String,
    pub pir_parameter_id: String,
    pub cryptographic_suite_id: String,
    pub epoch_policy: EpochPolicy,
    pub probes_per_operation: u32,
    pub slot_interval_ms: u64,
    pub request_bytes: u32,
    pub response_bytes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CompatibilityTuple {
    pub compatibility_policy_id: String,
    pub protocol_profile_id: String,
    pub corpus_release_id: String,
    pub pir_parameter_id: String,
    pub cryptographic_suite_id: String,
    pub epoch: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GovernancePolicy {
    RoutineThreeOfFive,
    CriticalFourOfFive,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SignatureEnvelope {
    pub policy: GovernancePolicy,
    pub key_id: String,
    pub payload_digest_sha256: String,
    pub signature: Vec<u8>,
}

impl ProtocolManifest {
    pub fn validate_structure_and_tuple(&self, tuple: &CompatibilityTuple) -> Result<(), String> {
        if self.schema_version != PROTOCOL_SCHEMA_VERSION {
            return Err("Unsupported protocol schema".into());
        }
        if [
            self.profile_id.as_str(),
            self.compatibility_policy_id.as_str(),
            self.corpus_release_id.as_str(),
            self.pir_parameter_id.as_str(),
            self.cryptographic_suite_id.as_str(),
        ]
        .iter()
        .any(|value| value.trim().is_empty())
        {
            return Err("Protocol identifiers must be non-empty".into());
        }
        if self.probes_per_operation == 0
            || self.slot_interval_ms == 0
            || self.request_bytes == 0
            || self.response_bytes == 0
        {
            return Err("Protocol geometry must be non-zero".into());
        }
        if self.probes_per_operation > u32::MAX / 2
            || self.slot_interval_ms > i64::MAX as u64
            || self.epoch_policy.maximum_clock_skew_ms > i64::MAX as u64
            || self.epoch_policy.maximum_clock_skew_ms >= self.slot_interval_ms
        {
            return Err("Protocol geometry is not representable".into());
        }
        if self.epoch_policy.activates_at_unix_ms >= self.epoch_policy.expires_at_unix_ms {
            return Err("Protocol epoch interval is invalid".into());
        }
        let expected = CompatibilityTuple {
            compatibility_policy_id: self.compatibility_policy_id.clone(),
            protocol_profile_id: self.profile_id.clone(),
            corpus_release_id: self.corpus_release_id.clone(),
            pir_parameter_id: self.pir_parameter_id.clone(),
            cryptographic_suite_id: self.cryptographic_suite_id.clone(),
            epoch: self.epoch_policy.epoch,
        };
        if &expected != tuple {
            return Err("Incompatible signed protocol tuple".into());
        }
        Ok(())
    }

    pub fn validate_active_time(&self, now_unix_ms: i64) -> Result<(), String> {
        if now_unix_ms < self.epoch_policy.activates_at_unix_ms
            || now_unix_ms >= self.epoch_policy.expires_at_unix_ms
        {
            return Err("Protocol epoch is not currently valid".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest() -> ProtocolManifest {
        ProtocolManifest {
            schema_version: PROTOCOL_SCHEMA_VERSION,
            profile_id: "lab-v1".into(),
            compatibility_policy_id: "clients-v1".into(),
            corpus_release_id: "corpus-v1".into(),
            pir_parameter_id: "pir-v1".into(),
            cryptographic_suite_id: "suite-v1".into(),
            epoch_policy: EpochPolicy {
                epoch: 7,
                activates_at_unix_ms: 1_000,
                expires_at_unix_ms: 2_000,
                maximum_clock_skew_ms: 100,
            },
            probes_per_operation: 2,
            slot_interval_ms: 10_000,
            request_bytes: 1024,
            response_bytes: 2048,
        }
    }

    #[test]
    fn validates_the_complete_tuple_and_epoch() {
        let manifest = manifest();
        let tuple = CompatibilityTuple {
            compatibility_policy_id: "clients-v1".into(),
            protocol_profile_id: "lab-v1".into(),
            corpus_release_id: "corpus-v1".into(),
            pir_parameter_id: "pir-v1".into(),
            cryptographic_suite_id: "suite-v1".into(),
            epoch: 7,
        };
        assert!(manifest.validate_structure_and_tuple(&tuple).is_ok());
        assert!(manifest.validate_active_time(1_500).is_ok());
        let mut incompatible = tuple.clone();
        incompatible.corpus_release_id = "stale".into();
        assert!(manifest
            .validate_structure_and_tuple(&incompatible)
            .is_err());
        assert!(manifest.validate_active_time(2_000).is_err());
    }

    #[test]
    fn rejects_unrepresentable_geometry() {
        let mut manifest = manifest();
        let tuple = CompatibilityTuple {
            compatibility_policy_id: "clients-v1".into(),
            protocol_profile_id: "lab-v1".into(),
            corpus_release_id: "corpus-v1".into(),
            pir_parameter_id: "pir-v1".into(),
            cryptographic_suite_id: "suite-v1".into(),
            epoch: 7,
        };
        manifest.slot_interval_ms = i64::MAX as u64 + 1;
        assert!(manifest.validate_structure_and_tuple(&tuple).is_err());
    }
}
