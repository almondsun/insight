use anyhow::{anyhow, bail, Context, Result};
use clap::{Parser, Subcommand};
use fame_core::{
    agent::{LabParticipationAgent, PrivateLookup},
    corpus::{build_synthetic_release, CorpusInputRecord},
    protocol::{CompatibilityTuple, ProtocolManifest},
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf, process::Command};

#[derive(Parser)]
#[command(about = "Synthetic-only feasibility harness; never a production Fame client")]
struct Cli {
    #[command(subcommand)]
    command: LabCommand,
}

#[derive(Subcommand)]
enum LabCommand {
    Status,
    Corpus {
        #[arg(long)]
        input: PathBuf,
    },
    Scheduler {
        #[arg(long)]
        manifest: PathBuf,
        #[arg(long)]
        tuple: PathBuf,
        #[arg(long)]
        username: Option<String>,
    },
    Bootstrap,
    Platform,
    Pir,
    Mixnet,
    Traffic {
        real: PathBuf,
        dummy: PathBuf,
    },
    Evidence {
        output: PathBuf,
    },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Gate<'a> {
    gate: &'a str,
    outcome: &'a str,
    detail: String,
}

fn available(program: &str, arg: &str) -> bool {
    Command::new(program)
        .arg(arg)
        .output()
        .is_ok_and(|result| result.status.success())
}

fn status() -> Vec<Gate<'static>> {
    vec![
        Gate { gate: "product_network_path", outcome: "PASS", detail: "No Fame network client is linked into the desktop application".into() },
        Gate { gate: "container_runtime", outcome: if available("docker", "version") { "AVAILABLE" } else { "BLOCKED" }, detail: "Required for the pinned Google DPF GCC 14 reproduction".into() },
        Gate { gate: "packet_capture", outcome: if available("tshark", "--version") { "AVAILABLE" } else { "BLOCKED" }, detail: "Required for preregistered live traffic capture".into() },
        Gate { gate: "production_pir", outcome: "BLOCKED", detail: "No audited deployable two-server PIR candidate or independent replica operators".into() },
        Gate { gate: "production_mixnet", outcome: "BLOCKED", detail: "No qualified pinned topology, operator-independence evidence, or traffic-analysis pass".into() },
        Gate { gate: "governance", outcome: "BLOCKED", detail: "Authority, witness, transparency, and fresh-time organizations are not established".into() },
    ]
}

fn read_json<T: serde::de::DeserializeOwned>(path: &PathBuf) -> Result<T> {
    serde_json::from_slice(&fs::read(path).with_context(|| format!("read {}", path.display()))?)
        .with_context(|| format!("parse {}", path.display()))
}

fn activation_times(activation: i64) -> Result<(i64, i64)> {
    let before = activation
        .checked_sub(1)
        .ok_or_else(|| anyhow!("manifest activation time cannot be represented safely"))?;
    Ok((before, activation))
}

fn main() -> Result<()> {
    let result = match Cli::parse().command {
        LabCommand::Status => serde_json::to_value(status())?,
        LabCommand::Corpus { input } => {
            let records: Vec<CorpusInputRecord> = read_json(&input)?;
            let release = build_synthetic_release("synthetic-r1", "generated-test-data", records)
                .map_err(|error| anyhow!(error))?;
            serde_json::json!({"outcome":"PASS","schemaVersion":release.schema_version(),"releaseId":release.release_id(),"records":release.records().len(),"recordBytes":64,"commitmentSha256":release.commitment_sha256()})
        }
        LabCommand::Scheduler {
            manifest,
            tuple,
            username,
        } => {
            let manifest: ProtocolManifest = read_json(&manifest)?;
            let tuple: CompatibilityTuple = read_json(&tuple)?;
            let (before, activation) =
                activation_times(manifest.epoch_policy.activates_at_unix_ms)?;
            let mut agent = LabParticipationAgent::default();
            agent
                .configure(manifest, &tuple, before)
                .map_err(|error| anyhow!(error))?;
            agent
                .activate_at_epoch_boundary(activation)
                .map_err(|error| anyhow!(error))?;
            if let Some(normalized_username) = username {
                agent
                    .enqueue(PrivateLookup {
                        normalized_username,
                    })
                    .map_err(|error| anyhow!(error))?;
            }
            let slot = agent
                .tick(activation)
                .context("scheduler did not emit at epoch boundary")?;
            serde_json::json!({"outcome":"PASS","slot":slot.slot,"scheduledAtUnixMs":slot.scheduled_at_unix_ms,"operationBytes":slot.operation.bytes().len(),"requests":slot.requests,"replyPaths":slot.reply_paths})
        }
        LabCommand::Bootstrap => {
            serde_json::json!({"outcome":"BLOCKED","reason":"Trust-root schemas are specified, but no real authority, witness, transparency-log, or fresh-time quorum exists"})
        }
        LabCommand::Platform => {
            serde_json::json!({"outcome":"BLOCKED","reason":"Strong rollback resistance requires per-platform restoration-detection evidence; this host is not classified by assumption"})
        }
        LabCommand::Pir => {
            serde_json::json!({"outcome":"BLOCKED","candidate":"google/distributed_point_functions@859cafa71fc1e139c7b76d4d4c0f23438688a8ad","reason":"Building a DPF primitive is not a deployable or audited two-server PIR service"})
        }
        LabCommand::Mixnet => {
            serde_json::json!({"outcome":"BLOCKED","primaryCandidate":"Katzenpost","comparisonCandidate":"Nym","reason":"Neither exact profile has passed the preregistered request/reply traffic experiment or operator-correlation review"})
        }
        LabCommand::Traffic { real, dummy } => {
            let real = fs::read(real)?;
            let dummy = fs::read(dummy)?;
            let exact = real == dummy;
            serde_json::json!({"outcome":"BLOCKED","smokeCheck":if exact{"EQUAL"}else{"DISTINGUISHABLE"},"deterministicByteEquality":exact,"realSha256":format!("{:x}",Sha256::digest(&real)),"dummySha256":format!("{:x}",Sha256::digest(&dummy)),"reason":"Byte equality is only a deterministic harness smoke check; the preregistered traffic-analysis evaluation is not implemented","productionPrivacyClaim":false})
        }
        LabCommand::Evidence { output } => {
            if output.exists() {
                bail!("refusing to overwrite existing evidence bundle")
            }
            let body = serde_json::json!({"schemaVersion":1,"generatedAt":chrono::Utc::now().to_rfc3339(),"syntheticOnly":true,"productionPrivacyClaim":false,"gates":status()});
            fs::write(&output, serde_json::to_vec_pretty(&body)?)?;
            serde_json::json!({"outcome":"PASS","written":output})
        }
    };
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::activation_times;

    #[test]
    fn rejects_unrepresentable_activation_predecessor() {
        assert!(activation_times(i64::MIN).is_err());
        assert_eq!(activation_times(0).unwrap(), (-1, 0));
    }
}
