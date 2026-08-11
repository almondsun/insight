use crate::protocol::{CompatibilityTuple, ProtocolManifest};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ParticipationState {
    Unconfigured,
    Bootstrap,
    WaitingForEpoch,
    Active,
    Suspended,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SuspensionReason {
    MissingQuorum,
    TimeUncertain,
    TransparencyInconsistent,
    IncompatibleRelease,
    PirUnavailable,
    MixnetUnavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrivateLookup {
    pub normalized_username: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabOperation {
    bytes: [u8; 32],
}

impl LabOperation {
    pub fn bytes(&self) -> &[u8; 32] {
        &self.bytes
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduledSlot {
    pub slot: u64,
    pub scheduled_at_unix_ms: i64,
    pub operation: LabOperation,
    pub requests: u32,
    pub reply_paths: u32,
}

pub struct LabParticipationAgent {
    state: ParticipationState,
    suspension_reason: Option<SuspensionReason>,
    manifest: Option<ProtocolManifest>,
    next_slot_at_unix_ms: Option<i64>,
    highest_accepted_epoch: Option<u64>,
    last_authenticated_time_unix_ms: Option<i64>,
    highest_observed_time_unix_ms: Option<i64>,
    queue: VecDeque<PrivateLookup>,
}

impl Default for LabParticipationAgent {
    fn default() -> Self {
        Self {
            state: ParticipationState::Unconfigured,
            suspension_reason: None,
            manifest: None,
            next_slot_at_unix_ms: None,
            highest_accepted_epoch: None,
            last_authenticated_time_unix_ms: None,
            highest_observed_time_unix_ms: None,
            queue: VecDeque::new(),
        }
    }
}

impl LabParticipationAgent {
    pub fn state(&self) -> &ParticipationState {
        &self.state
    }

    pub fn suspension_reason(&self) -> Option<&SuspensionReason> {
        self.suspension_reason.as_ref()
    }

    pub fn pending_lookup_count(&self) -> usize {
        self.queue.len()
    }

    pub fn configure(
        &mut self,
        manifest: ProtocolManifest,
        tuple: &CompatibilityTuple,
        authenticated_now_unix_ms: i64,
    ) -> Result<(), String> {
        if !matches!(
            self.state,
            ParticipationState::Unconfigured | ParticipationState::Suspended
        ) {
            return Err("Agent cannot be configured from its current state".into());
        }
        manifest.validate_structure_and_tuple(tuple)?;
        if authenticated_now_unix_ms >= manifest.epoch_policy.activates_at_unix_ms {
            return Err("Protocol epoch must be staged before its activation boundary".into());
        }
        if self
            .last_authenticated_time_unix_ms
            .is_some_and(|previous| authenticated_now_unix_ms <= previous)
            || self
                .highest_observed_time_unix_ms
                .is_some_and(|previous| authenticated_now_unix_ms <= previous)
        {
            return Err("Authenticated time did not advance".into());
        }
        if self
            .highest_observed_time_unix_ms
            .is_some_and(|previous| manifest.epoch_policy.activates_at_unix_ms <= previous)
        {
            return Err("Protocol activation boundary would roll back local time".into());
        }
        if self
            .highest_accepted_epoch
            .is_some_and(|epoch| manifest.epoch_policy.epoch <= epoch)
        {
            return Err("Protocol epoch would replay or roll back state".into());
        }
        self.highest_accepted_epoch = Some(manifest.epoch_policy.epoch);
        self.next_slot_at_unix_ms = Some(manifest.epoch_policy.activates_at_unix_ms);
        self.manifest = Some(manifest);
        self.last_authenticated_time_unix_ms = Some(authenticated_now_unix_ms);
        self.highest_observed_time_unix_ms = Some(authenticated_now_unix_ms);
        self.suspension_reason = None;
        self.state = ParticipationState::WaitingForEpoch;
        Ok(())
    }

    pub fn activate_at_epoch_boundary(
        &mut self,
        authenticated_now_unix_ms: i64,
    ) -> Result<(), String> {
        if self.state != ParticipationState::WaitingForEpoch {
            return Err("Agent is not waiting for an epoch boundary".into());
        }
        let manifest = self.manifest.as_ref().ok_or("Agent is not configured")?;
        if authenticated_now_unix_ms != manifest.epoch_policy.activates_at_unix_ms {
            return Err("Participation may activate only at the epoch boundary".into());
        }
        if self
            .last_authenticated_time_unix_ms
            .is_some_and(|previous| authenticated_now_unix_ms <= previous)
        {
            return Err("Authenticated time did not advance".into());
        }
        manifest.validate_active_time(authenticated_now_unix_ms)?;
        self.last_authenticated_time_unix_ms = Some(authenticated_now_unix_ms);
        self.highest_observed_time_unix_ms = Some(authenticated_now_unix_ms);
        self.next_slot_at_unix_ms = Some(authenticated_now_unix_ms);
        self.state = ParticipationState::Active;
        Ok(())
    }

    pub fn enqueue(&mut self, lookup: PrivateLookup) -> Result<(), String> {
        let canonical = crate::identity::normalize(&lookup.normalized_username);
        if !crate::identity::is_valid_username(&lookup.normalized_username)
            || lookup.normalized_username != canonical
        {
            return Err("Lookup username must be normalized".into());
        }
        self.queue.push_back(lookup);
        Ok(())
    }

    pub fn tick(&mut self, now_unix_ms: i64) -> Option<ScheduledSlot> {
        if self.state != ParticipationState::Active {
            return None;
        }
        if self
            .highest_observed_time_unix_ms
            .is_some_and(|previous| now_unix_ms < previous)
        {
            self.suspend(SuspensionReason::TimeUncertain, now_unix_ms);
            return None;
        }
        self.highest_observed_time_unix_ms = Some(now_unix_ms);
        let manifest = self.manifest.as_ref()?;
        let expiry = manifest.epoch_policy.expires_at_unix_ms;
        let maximum_lateness_raw = manifest.epoch_policy.maximum_clock_skew_ms;
        let interval_raw = manifest.slot_interval_ms;
        let probes_raw = manifest.probes_per_operation;

        if now_unix_ms >= expiry {
            self.suspend(SuspensionReason::TimeUncertain, now_unix_ms);
            return None;
        }
        let next_due = self.next_slot_at_unix_ms?;
        if now_unix_ms < next_due {
            return None;
        }

        let computed = (|| {
            let maximum_lateness = i64::try_from(maximum_lateness_raw).ok()?;
            let interval = i64::try_from(interval_raw).ok()?;
            let probes = probes_raw.checked_mul(2)?;
            let lateness = now_unix_ms.checked_sub(next_due)?;
            let activation = self.manifest.as_ref()?.epoch_policy.activates_at_unix_ms;
            let elapsed = next_due.checked_sub(activation)?;
            let slot_i64 = elapsed.checked_div(interval)?;
            let slot = u64::try_from(slot_i64).ok()?;
            let next_boundary = next_due.checked_add(interval)?;
            Some((slot, lateness, maximum_lateness, next_boundary, probes))
        })();
        let Some((slot, lateness, maximum_lateness, next_boundary, probes)) = computed else {
            self.suspend(SuspensionReason::IncompatibleRelease, now_unix_ms);
            return None;
        };
        if lateness > maximum_lateness {
            self.suspend(SuspensionReason::TimeUncertain, now_unix_ms);
            return None;
        }
        self.next_slot_at_unix_ms = Some(next_boundary);

        let lookup = self.queue.pop_front();
        let operation = make_lab_operation(slot, lookup.as_ref());
        Some(ScheduledSlot {
            slot,
            scheduled_at_unix_ms: next_due,
            operation,
            requests: probes,
            reply_paths: probes,
        })
    }

    pub fn suspend(&mut self, reason: SuspensionReason, observed_now_unix_ms: i64) {
        self.highest_observed_time_unix_ms = Some(
            self.highest_observed_time_unix_ms
                .map_or(observed_now_unix_ms, |previous| {
                    previous.max(observed_now_unix_ms)
                }),
        );
        self.state = ParticipationState::Suspended;
        self.suspension_reason = Some(reason);
        self.next_slot_at_unix_ms = None;
    }
}

fn make_lab_operation(slot: u64, lookup: Option<&PrivateLookup>) -> LabOperation {
    let mut hasher = Sha256::new();
    hasher.update(b"nivune-lab-operation-v1\0");
    hasher.update(slot.to_be_bytes());
    match lookup {
        Some(lookup) => {
            hasher.update([1]);
            hasher.update(lookup.normalized_username.as_bytes());
        }
        None => hasher.update([0]),
    }
    LabOperation {
        bytes: hasher.finalize().into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{EpochPolicy, PROTOCOL_SCHEMA_VERSION};

    fn profile() -> (ProtocolManifest, CompatibilityTuple) {
        let manifest = ProtocolManifest {
            schema_version: PROTOCOL_SCHEMA_VERSION,
            profile_id: "lab".into(),
            compatibility_policy_id: "clients".into(),
            corpus_release_id: "corpus".into(),
            pir_parameter_id: "pir".into(),
            cryptographic_suite_id: "suite".into(),
            epoch_policy: EpochPolicy {
                epoch: 1,
                activates_at_unix_ms: 1_000,
                expires_at_unix_ms: 10_000,
                maximum_clock_skew_ms: 100,
            },
            probes_per_operation: 2,
            slot_interval_ms: 1_000,
            request_bytes: 1024,
            response_bytes: 2048,
        };
        let tuple = CompatibilityTuple {
            compatibility_policy_id: "clients".into(),
            protocol_profile_id: "lab".into(),
            corpus_release_id: "corpus".into(),
            pir_parameter_id: "pir".into(),
            cryptographic_suite_id: "suite".into(),
            epoch: 1,
        };
        (manifest, tuple)
    }

    fn active_agent() -> LabParticipationAgent {
        let (manifest, tuple) = profile();
        let mut agent = LabParticipationAgent::default();
        agent.configure(manifest, &tuple, 900).unwrap();
        agent.activate_at_epoch_boundary(1_000).unwrap();
        agent
    }

    #[test]
    fn substitutes_real_work_without_changing_slot_shape() {
        let mut agent = active_agent();
        let dummy = agent.tick(1_000).unwrap();
        agent
            .enqueue(PrivateLookup {
                normalized_username: "alice".into(),
            })
            .unwrap();
        let real = agent.tick(2_000).unwrap();
        assert_eq!(
            (
                dummy.requests,
                dummy.reply_paths,
                dummy.operation.bytes().len()
            ),
            (
                real.requests,
                real.reply_paths,
                real.operation.bytes().len()
            )
        );
        assert_eq!(agent.pending_lookup_count(), 0);
    }

    #[test]
    fn missed_slots_are_not_replayed_or_caught_up() {
        let mut agent = active_agent();
        assert!(agent.tick(4_500).is_none());
        assert_eq!(agent.state(), &ParticipationState::Suspended);
        assert_eq!(
            agent.suspension_reason(),
            Some(&SuspensionReason::TimeUncertain)
        );
    }

    #[test]
    fn slight_scheduler_jitter_emits_only_the_expected_slot() {
        let mut agent = active_agent();
        let current = agent.tick(1_050).unwrap();
        assert_eq!(current.slot, 0);
        assert_eq!(current.scheduled_at_unix_ms, 1_000);
        assert!(agent.tick(1_050).is_none());
    }

    #[test]
    fn suspension_requires_a_newer_staged_epoch() {
        let (manifest, tuple) = profile();
        let mut agent = active_agent();
        agent.suspend(SuspensionReason::PirUnavailable, 1_500);
        assert!(agent.activate_at_epoch_boundary(1_000).is_err());
        assert!(agent.configure(manifest.clone(), &tuple, 1_100).is_err());

        let mut next = manifest;
        next.epoch_policy.epoch = 2;
        next.epoch_policy.activates_at_unix_ms = 3_000;
        next.epoch_policy.expires_at_unix_ms = 12_000;
        let mut next_tuple = tuple;
        next_tuple.epoch = 2;
        assert!(agent.configure(next, &next_tuple, 2_500).is_ok());
    }

    #[test]
    fn newer_epoch_cannot_reactivate_in_chronological_past() {
        let (manifest, tuple) = profile();
        let mut agent = active_agent();
        agent.suspend(SuspensionReason::PirUnavailable, 9_100);

        let mut next = manifest;
        next.epoch_policy.epoch = 2;
        next.epoch_policy.activates_at_unix_ms = 2_000;
        next.epoch_policy.expires_at_unix_ms = 12_000;
        let mut next_tuple = tuple;
        next_tuple.epoch = 2;
        assert!(agent.configure(next, &next_tuple, 1_500).is_err());
    }

    #[test]
    fn highest_staged_epoch_cannot_be_replaced_by_a_lower_epoch() {
        let (mut manifest, mut tuple) = profile();
        manifest.epoch_policy.epoch = 5;
        tuple.epoch = 5;
        let mut agent = LabParticipationAgent::default();
        agent.configure(manifest.clone(), &tuple, 900).unwrap();
        agent.suspend(SuspensionReason::MissingQuorum, 950);

        manifest.epoch_policy.epoch = 4;
        manifest.epoch_policy.activates_at_unix_ms = 2_000;
        tuple.epoch = 4;
        assert!(agent.configure(manifest, &tuple, 1_500).is_err());
    }

    #[test]
    fn rejects_noncanonical_lookup() {
        let mut agent = active_agent();
        assert!(agent
            .enqueue(PrivateLookup {
                normalized_username: " Alice ".into(),
            })
            .is_err());
    }
}
