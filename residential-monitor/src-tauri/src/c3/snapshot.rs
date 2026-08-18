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
        if self.total_bytes.saturating_add(bytes) > MAX_SPOOL_BYTES {
            return Err(ReportError::QuotaExceeded("spool quota"));
        }
        if self.items.len() >= MAX_ACTIVE_TOKENS {
            return Err(ReportError::QuotaExceeded("active token count"));
        }
        let token = new_token();
        let checksum = hex::encode(Sha256::digest(&encoded));
        let spool_path = self.spool_dir.join(format!("{token}.json"));
        std::fs::write(&spool_path, &encoded).map_err(|_| ReportError::Failed("spool write"))?;
        result.report_snapshot_token = token.clone();
        let record = SnapshotRecord {
            token: token.clone(),
            fingerprint: query_fingerprint(query),
            schema_version: result.schema_version,
            data_version: result.data_version,
            created_utc: now_utc,
            expires_utc: now_utc + TOKEN_TTL_SECS as i64,
            bytes,
            checksum,
            spool_path: Some(spool_path),
            result: result.clone(),
        };
        self.total_bytes = self.total_bytes.saturating_add(bytes);
        self.items.insert(token, record);
        Ok(result)
    }

    pub fn get(&self, token: &str, now_utc: i64) -> Result<&ReportResult, ReportError> {
        let item = self
            .items
            .get(token)
            .ok_or(ReportError::TokenExpired("missing"))?;
        if item.expires_utc <= now_utc {
            return Err(ReportError::TokenExpired("ttl"));
        }
        Ok(&item.result)
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

    #[test]
    fn rejects_open_transaction_and_quota() {
        let dir = tempfile::tempdir().expect("dir");
        let mut store = ReportSnapshotStore::open(dir.path());
        let query = ReportQuery::default();
        let plan = plan_capability(&query, 4_000, 30).expect("plan");
        let result = empty_result(query.clone(), &plan, 1);
        let error = store
            .insert(&query, result.clone(), 100, true)
            .expect_err("txn");
        assert_eq!(error.code(), "storage_failure");
        for index in 0..MAX_ACTIVE_TOKENS {
            let mut query = ReportQuery::default();
            query.range_start_utc = i64::from(index as u32) * 10;
            query.range_end_utc = query.range_start_utc + 10;
            let plan = plan_capability(&query, 4_000, 30).expect("plan");
            store
                .insert(&query, empty_result(query.clone(), &plan, 1), 100, false)
                .expect("insert");
        }
        let extra = store.insert(&query, result, 100, false).expect_err("quota");
        assert_eq!(extra.code(), "quota_exceeded");
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
