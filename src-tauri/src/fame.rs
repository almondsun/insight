use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

pub const FORMULA_VERSION: &str = "fame-v1";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Precision {
    Exact,
    Approximate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FameObservation {
    pub username: String,
    pub normalized_username: String,
    pub followers: u64,
    pub following: u64,
    pub precision: Precision,
    pub observed_at: String,
    pub source: String,
    pub corpus_release: String,
    pub authenticated: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FameRankedRow {
    pub rank: usize,
    pub username: String,
    pub normalized_username: String,
    pub followers: u64,
    pub following: u64,
    pub score: f64,
    pub precision: Precision,
    pub observed_at: String,
    pub source: String,
    pub corpus_release: String,
    pub formula_version: &'static str,
}

pub fn score(followers: u64, following: u64) -> f64 {
    if followers <= following {
        return 0.0;
    }
    let f = followers as f64;
    let g = following as f64;
    let total = f + g;
    let follower_term = if followers == 0 {
        0.0
    } else {
        f * (2.0 * f / total).log2()
    };
    let following_term = if following == 0 {
        0.0
    } else {
        g * (2.0 * g / total).log2()
    };
    let value = follower_term + following_term;
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

pub fn rank(observations: Vec<FameObservation>) -> Result<Vec<FameRankedRow>, String> {
    if observations
        .iter()
        .any(|observation| !observation.authenticated)
    {
        return Err("Unauthenticated observations cannot be ranked".into());
    }
    let mut rows = observations
        .into_iter()
        .map(|observation| {
            let value = score(observation.followers, observation.following);
            (observation, value)
        })
        .collect::<Vec<_>>();
    rows.sort_by(|(left, left_score), (right, right_score)| {
        right_score
            .partial_cmp(left_score)
            .unwrap_or(Ordering::Equal)
            .then_with(|| right.followers.cmp(&left.followers))
            .then_with(|| left.normalized_username.cmp(&right.normalized_username))
    });
    Ok(rows
        .into_iter()
        .enumerate()
        .map(|(index, (observation, value))| FameRankedRow {
            rank: index + 1,
            username: observation.username,
            normalized_username: observation.normalized_username,
            followers: observation.followers,
            following: observation.following,
            score: value,
            precision: observation.precision,
            observed_at: observation.observed_at,
            source: observation.source,
            corpus_release: observation.corpus_release,
            formula_version: FORMULA_VERSION,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formula_examples_and_boundaries() {
        assert_eq!(score(10, 10), 0.0);
        assert_eq!(score(0, 0), 0.0);
        assert_eq!(score(1, 2), 0.0);
        assert_eq!(score(100, 0), 100.0);
        assert!((score(100, 10) - 61.655_331).abs() < 0.000_001);
        assert!(score(u64::MAX, 0).is_finite());
    }

    #[test]
    fn ranking_is_deterministic() {
        let observation = |username: &str, followers, following| FameObservation {
            username: username.into(),
            normalized_username: username.to_lowercase(),
            followers,
            following,
            precision: Precision::Exact,
            observed_at: "2026-01-01T00:00:00Z".into(),
            source: "test".into(),
            corpus_release: "r1".into(),
            authenticated: true,
        };
        let rows = rank(vec![
            observation("zoe", 10, 10),
            observation("Alice", 10, 10),
            observation("bob", 100, 0),
        ])
        .unwrap();
        assert_eq!(
            rows.iter()
                .map(|row| row.username.as_str())
                .collect::<Vec<_>>(),
            vec!["bob", "Alice", "zoe"]
        );
        assert_eq!(rows[0].formula_version, FORMULA_VERSION);
    }

    #[test]
    fn rejects_unauthenticated_observations() {
        let result = rank(vec![FameObservation {
            username: "alice".into(),
            normalized_username: "alice".into(),
            followers: 10,
            following: 1,
            precision: Precision::Exact,
            observed_at: "2026-01-01T00:00:00Z".into(),
            source: "test".into(),
            corpus_release: "r1".into(),
            authenticated: false,
        }]);
        assert!(result.is_err());
    }
}
