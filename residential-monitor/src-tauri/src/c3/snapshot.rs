//! 报告快照：token 不持有 SQLite 读事务。

use crate::c3::query::{
    query_fingerprint, ReportError, ReportQuery, ReportResult, MAX_ACTIVE_TOKENS, MAX_SPOOL_BYTES,
    MAX_TOKEN_BYTES, TOKEN_TTL_SECS,
};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct SnapshotRecord {
    pub token: String,
    pub fingerprint: String,
    pub schema_version: u32,
    pub data_version: u64,
    pub created_utc: i64,
    pub expires_utc: i64,
    pub last_access_utc: i64,
    pub bytes: u64,
    pub checksum: String,
    pub spool_path: Option<PathBuf>,
    pub result: ReportResult,
}

pub struct ReportSnapshotStore {
    items: HashMap<String, SnapshotRecord>,
    spool_dir: PathBuf,
    total_bytes: u64,
}

impl ReportSnapshotStore {
    pub fn open(data_dir: impl AsRef<Path>) -> Self {
        let spool_dir = data_dir.as_ref().join("report-spool");
        let _ = std::fs::create_dir_all(&spool_dir);
        let mut store = Self {
            items: HashMap::new(),
            spool_dir,
            total_bytes: 0,
        };
        store.cleanup_orphans();
        store
    }

    pub fn cleanup_expired(&mut self, now_utc: i64) {
        let expired: Vec<String> = self
            .items
            .iter()
            .filter(|(_, item)| item.expires_utc <= now_utc)
            .map(|(token, _)| token.clone())
            .collect();
        for token in expired {
            self.release(&token);
        }
    }

    pub fn cleanup_orphans(&mut self) {
        if let Ok(entries) = std::fs::read_dir(&self.spool_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|item| item.to_str()) == Some("json") {
                    let stem = path
                        .file_stem()
                        .and_then(|item| item.to_str())
                        .unwrap_or_default();
                    if !self.items.contains_key(stem) {
                        let _ = std::fs::remove_file(path);
                    }
                }
            }
        }
    }

    pub fn insert(
        &mut self,
        query: &ReportQuery,
        mut result: ReportResult,
        now_utc: i64,
        read_txn_open: bool,
    ) -> Result<ReportResult, ReportError> {
        if read_txn_open {
            return Err(ReportError::Failed(
                "read transaction still open before token return",
            ));
        }
        self.cleanup_expired(now_utc);
        let encoded = serde_json::to_vec(&result).map_err(|_| ReportError::Failed("encode"))?;
        let bytes = encoded.len() as u64;
        if bytes > MAX_TOKEN_BYTES {
            return Err(ReportError::QuotaExceeded("single token too large"));
        }
        let fingerprint = query_fingerprint(query);
        if let Some(token) = self.live_token_for(&fingerprint, now_utc) {
            return self.replace_token(&token, result, encoded, bytes, now_utc, fingerprint);
        }
        self.evict_lru_until_fits(bytes);
        if self.items.len() >= MAX_ACTIVE_TOKENS
            || self.total_bytes.saturating_add(bytes) > MAX_SPOOL_BYTES
        {
            return Err(ReportError::QuotaExceeded("spool quota"));
        }
        let token = new_token();
        let checksum = hex::encode(Sha256::digest(&encoded));
        let spool_path = self.spool_dir.join(format!("{token}.json"));
        std::fs::write(&spool_path, &encoded).map_err(|_| ReportError::Failed("spool write"))?;
        result.report_snapshot_token = token.clone();
        let record = SnapshotRecord {
            token: token.clone(),
            fingerprint,
            schema_version: result.schema_version,
            data_version: result.data_version,
            created_utc: now_utc,
            expires_utc: now_utc + TOKEN_TTL_SECS as i64,
            last_access_utc: now_utc,
            bytes,
            checksum,
            spool_path: Some(spool_path),
            result: result.clone(),
        };
        self.total_bytes = self.total_bytes.saturating_add(bytes);
        self.items.insert(token, record);
        Ok(result)
    }

    pub fn get(&mut self, token: &str, now_utc: i64) -> Result<&ReportResult, ReportError> {
        let item = self
            .items
            .get_mut(token)
            .ok_or(ReportError::TokenExpired("missing"))?;
        if item.expires_utc <= now_utc {
            return Err(ReportError::TokenExpired("ttl"));
        }
        item.last_access_utc = now_utc;
        Ok(&item.result)
    }

    fn live_token_for(&self, fingerprint: &str, now_utc: i64) -> Option<String> {
        self.items
            .values()
            .find(|item| item.fingerprint == fingerprint && item.expires_utc > now_utc)
            .map(|item| item.token.clone())
    }

    fn replace_token(
        &mut self,
        token: &str,
        mut result: ReportResult,
        encoded: Vec<u8>,
        bytes: u64,
        now_utc: i64,
        fingerprint: String,
    ) -> Result<ReportResult, ReportError> {
        let Some(existing) = self.items.get(token) else {
            return Err(ReportError::TokenExpired("missing"));
        };
        let spool_path = existing
            .spool_path
            .clone()
            .unwrap_or_else(|| self.spool_dir.join(format!("{token}.json")));
        let old_bytes = existing.bytes;
        let created_utc = existing.created_utc;
        std::fs::write(&spool_path, &encoded).map_err(|_| ReportError::Failed("spool write"))?;
        self.total_bytes = self
            .total_bytes
            .saturating_sub(old_bytes)
            .saturating_add(bytes);
        result.report_snapshot_token = token.to_string();
        if let Some(record) = self.items.get_mut(token) {
            record.fingerprint = fingerprint;
            record.schema_version = result.schema_version;
            record.data_version = result.data_version;
            record.created_utc = created_utc;
            record.expires_utc = now_utc + TOKEN_TTL_SECS as i64;
            record.last_access_utc = now_utc;
            record.bytes = bytes;
            record.checksum = hex::encode(Sha256::digest(&encoded));
            record.spool_path = Some(spool_path);
            record.result = result.clone();
        }
        Ok(result)
    }

    fn evict_lru_until_fits(&mut self, extra_bytes: u64) {
        while self.items.len() >= MAX_ACTIVE_TOKENS
            || self.total_bytes.saturating_add(extra_bytes) > MAX_SPOOL_BYTES
        {
            let victim = self
                .items
                .values()
                .min_by_key(|item| (item.last_access_utc, item.created_utc, item.token.as_str()))
                .map(|item| item.token.clone());
            let Some(token) = victim else {
                break;
            };
            self.release(&token);
        }
    }

    pub fn release(&mut self, token: &str) -> bool {
        if let Some(item) = self.items.remove(token) {
            self.total_bytes = self.total_bytes.saturating_sub(item.bytes);
            if let Some(path) = item.spool_path {
                let _ = std::fs::remove_file(path);
            }
            return true;
        }
        false
    }

    pub fn active_count(&self) -> usize {
        self.items.len()
    }

    pub fn total_bytes(&self) -> u64 {
        self.total_bytes
    }
}

fn new_token() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|item| item.as_nanos())
        .unwrap_or(0);
    hex::encode(Sha256::digest(format!("c3-token-{nanos}").as_bytes()))[..32].to_string()
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod snapshot_store_tests {
    use super::*;
    use crate::c3::query::{empty_result, plan_capability, ReportQuery};

    fn sample(range_start: i64) -> (ReportQuery, ReportResult) {
        let mut query = ReportQuery::default();
        query.range_start_utc = range_start;
        query.range_end_utc = range_start + 10;
        let plan = plan_capability(&query, 4_000, 30).expect("plan");
        let result = empty_result(query.clone(), &plan, 1);
        (query, result)
    }

    #[test]
    fn rejects_open_transaction() {
        let dir = tempfile::tempdir().expect("dir");
        let mut store = ReportSnapshotStore::open(dir.path());
        let (query, result) = sample(0);
        let error = store.insert(&query, result, 100, true).expect_err("txn");
        assert_eq!(error.code(), "storage_failure");
    }

    #[test]
    fn reuses_unexpired_fingerprint() {
        let dir = tempfile::tempdir().expect("dir");
        let mut store = ReportSnapshotStore::open(dir.path());
        let (query, result) = sample(0);
        let first = store.insert(&query, result, 100, false).expect("first");
        let plan = plan_capability(&query, 4_000, 30).expect("plan");
        let mut newer = empty_result(query.clone(), &plan, 2);
        newer.data_version = 2;
        let second = store.insert(&query, newer, 110, false).expect("reuse");
        assert_eq!(first.report_snapshot_token, second.report_snapshot_token);
        assert_eq!(store.active_count(), 1);
        assert_eq!(second.data_version, 2);
    }

    #[test]
    fn ninth_insert_evicts_oldest_access() {
        let dir = tempfile::tempdir().expect("dir");
        let mut store = ReportSnapshotStore::open(dir.path());
        let mut tokens = Vec::new();
        for index in 0..MAX_ACTIVE_TOKENS {
            let (query, result) = sample(i64::from(index as u32) * 10);
            let stored = store.insert(&query, result, 100, false).expect("insert");
            tokens.push(stored.report_snapshot_token);
        }
        store
            .get(&tokens[0], 101)
            .expect("pin oldest-created as newest access");
        let (query, result) = sample(9_000);
        let extra = store.insert(&query, result, 102, false).expect("lru");
        assert_eq!(store.active_count(), MAX_ACTIVE_TOKENS);
        assert!(store.get(&tokens[0], 102).is_ok());
        assert!(store.get(&extra.report_snapshot_token, 102).is_ok());
        let missing = tokens
            .iter()
            .skip(1)
            .filter(|token| store.get(token, 102).is_err())
            .count();
        assert_eq!(missing, 1);
    }

    #[test]
    fn rejects_single_token_over_max_bytes() {
        let dir = tempfile::tempdir().expect("dir");
        let mut store = ReportSnapshotStore::open(dir.path());
        let (mut query, mut result) = sample(0);
        result.policy_metadata.note_zh = "x".repeat((MAX_TOKEN_BYTES as usize) + 8);
        query.range_start_utc = 1;
        let error = store
            .insert(&query, result, 100, false)
            .expect_err("too large");
        assert_eq!(error.code(), "quota_exceeded");
        assert_eq!(store.active_count(), 0);
    }

    #[test]
    fn expired_token_requires_rerun() {
        let dir = tempfile::tempdir().expect("dir");
        let mut store = ReportSnapshotStore::open(dir.path());
        let query = ReportQuery::default();
        let plan = plan_capability(&query, 4_000, 30).expect("plan");
        let stored = store
            .insert(&query, empty_result(query.clone(), &plan, 1), 10, false)
            .expect("insert");
        store.cleanup_expired(10 + TOKEN_TTL_SECS as i64 + 1);
        let error = store
            .get(
                &stored.report_snapshot_token,
                10 + TOKEN_TTL_SECS as i64 + 1,
            )
            .expect_err("expired");
        assert_eq!(error.code(), "token_expired");
    }
}
