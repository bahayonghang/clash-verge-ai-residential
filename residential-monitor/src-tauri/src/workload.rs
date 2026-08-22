//! 与物理 schema 解耦的固定工作负载协议。

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const PROTOCOL_VERSION: u32 = 1;
pub const DEFAULT_SEED: u64 = 0x524d_4e54_4330_0001;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkloadSpec {
    pub version: u32,
    pub seed: u64,
    pub average_active: u32,
    pub mean_session_minutes: f64,
    pub mean_chain_nodes: f64,
    pub nonzero_minute_ratio: f64,
    pub days: u32,
    pub sample_hz: u32,
    pub domain_cardinality: u32,
    pub process_cardinality: u32,
    pub rule_cardinality: u32,
    pub chain_cardinality: u32,
    pub network_cardinality: u32,
    pub frame_change_ratio: f64,
    pub peak_active: u32,
    pub peak_minutes: u32,
    pub long_process_path: bool,
    pub hostile_domain_mix: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExpectedCounts {
    pub duration_minutes: u64,
    pub session_rows: u64,
    pub chain_rows: u64,
    pub minute_rows: u64,
    pub hourly_rows: u64,
    pub daily_rows: u64,
}

impl WorkloadSpec {
    pub fn profile_full(average_active: u32, days: u32) -> Self {
        Self {
            version: PROTOCOL_VERSION,
            seed: DEFAULT_SEED,
            average_active,
            mean_session_minutes: 5.0,
            mean_chain_nodes: 3.0,
            nonzero_minute_ratio: 1.0,
            days,
            sample_hz: 1,
            domain_cardinality: 800,
            process_cardinality: 120,
            rule_cardinality: 40,
            chain_cardinality: 60,
            network_cardinality: 4,
            frame_change_ratio: 1.0,
            peak_active: 10_000,
            peak_minutes: 30,
            long_process_path: true,
            hostile_domain_mix: true,
        }
    }

    pub fn smoke() -> Self {
        Self {
            version: PROTOCOL_VERSION,
            seed: DEFAULT_SEED,
            average_active: 2,
            mean_session_minutes: 2.0,
            mean_chain_nodes: 2.0,
            nonzero_minute_ratio: 1.0,
            days: 0,
            sample_hz: 1,
            domain_cardinality: 4,
            process_cardinality: 3,
            rule_cardinality: 2,
            chain_cardinality: 2,
            network_cardinality: 2,
            frame_change_ratio: 1.0,
            peak_active: 8,
            peak_minutes: 1,
            long_process_path: true,
            hostile_domain_mix: true,
        }
    }

    pub fn peak() -> Self {
        let mut spec = Self::profile_full(250, 30);
        spec.peak_active = 10_000;
        spec.peak_minutes = 30;
        spec.frame_change_ratio = 1.0;
        spec
    }

    pub fn duration_minutes(&self) -> u64 {
        if self.days == 0 {
            3
        } else {
            u64::from(self.days) * 1_440
        }
    }

    pub fn expected_counts(&self) -> ExpectedCounts {
        let duration_minutes = self.duration_minutes();
        let active = u64::from(self.average_active);
        let session_rows =
            ((active * duration_minutes) as f64 / self.mean_session_minutes).round() as u64;
        let chain_rows = (session_rows as f64 * self.mean_chain_nodes).round() as u64;
        let minute_rows =
            ((active * duration_minutes) as f64 * self.nonzero_minute_ratio).round() as u64;
        let hourly_buckets = duration_minutes.div_ceil(60).max(1);
        let dim_keys = u64::from(
            self.domain_cardinality
                + self.process_cardinality
                + self.rule_cardinality
                + self.chain_cardinality
                + self.network_cardinality,
        );
        let hourly_rows = hourly_buckets * dim_keys.min(32);
        let daily_rows = duration_minutes.div_ceil(1_440).max(1);
        ExpectedCounts {
            duration_minutes,
            session_rows,
            chain_rows,
            minute_rows,
            hourly_rows,
            daily_rows,
        }
    }

    pub fn manifest_hash(&self) -> String {
        let payload = serde_json::to_vec(self).expect("WorkloadSpec 可序列化");
        let digest = Sha256::digest(payload);
        hex::encode(digest)
    }
}

#[cfg(test)]
mod workload_spec_tests {
    use super::*;

    #[test]
    fn workload_spec_hash_is_stable_for_same_seed() {
        let left = WorkloadSpec::profile_full(50, 30);
        let right = WorkloadSpec::profile_full(50, 30);
        assert_eq!(left.manifest_hash(), right.manifest_hash());
        assert_eq!(left.expected_counts().duration_minutes, 43_200);
        assert_eq!(left.expected_counts().minute_rows, 2_160_000);
        assert_eq!(left.expected_counts().session_rows, 432_000);
        assert_eq!(left.expected_counts().chain_rows, 1_296_000);
    }

    #[test]
    fn workload_spec_smoke_stays_tiny() {
        let smoke = WorkloadSpec::smoke();
        let counts = smoke.expected_counts();
        assert_eq!(counts.duration_minutes, 3);
        assert!(counts.minute_rows <= 8);
        assert_eq!(smoke.manifest_hash().len(), 64);
    }
}
