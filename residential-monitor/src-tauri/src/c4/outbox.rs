//! 可靠 notification outbox：lease、stale reclaim、有上限退避。

use crate::c4::notify::{NotificationSink, NotifyError, NotifyPayload};
use crate::c4::schema::OUTBOX_SCAN_LIMIT;
use crate::c4::types::{OutboxIntent, OutboxStatus};
use crate::storage::{StorageCoordinator, StorageError};
use rusqlite::{params, OptionalExtension};
use sha2::{Digest, Sha256};

pub const LEASE_SECS: i64 = 30;
pub const MAX_ATTEMPTS: i64 = 8;
pub const BACKOFF_CAP_SECS: i64 = 3_600;

pub const SQL_OUTBOX_DUE: &str = "
select outbox_id, event_id, bundle_id, status, attempt, next_attempt_at,
       lease_until, lease_token, error_class, error_summary, idempotency_key, created_utc
  from notification_outbox
 where status in ('pending', 'retry')
   and next_attempt_at <= ?1
 order by next_attempt_at, outbox_id
 limit ?2
";

pub const SQL_OUTBOX_STALE: &str = "
select outbox_id
  from notification_outbox
 where status = 'leased'
   and lease_until is not null
   and lease_until < ?1
 order by lease_until, outbox_id
 limit ?2
";

pub fn persist_intents(
    coordinator: &StorageCoordinator,
    intents: &[OutboxIntent],
) -> Result<(), StorageError> {
    for item in intents {
        coordinator.connection().execute(
            "insert or ignore into notification_outbox(
                outbox_id, event_id, bundle_id, status, attempt, next_attempt_at,
                lease_until, lease_token, error_class, error_summary, idempotency_key, created_utc
             ) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                item.outbox_id,
                item.event_id,
                item.bundle_id,
                status_sql(item.status),
                item.attempt,
                item.next_attempt_at,
                item.lease_until,
                item.lease_token,
                item.error_class,
                item.error_summary,
                item.idempotency_key,
                item.created_utc
            ],
        )?;
    }
    Ok(())
}

pub fn reclaim_stale(coordinator: &StorageCoordinator, now_utc: i64) -> Result<u32, StorageError> {
    let ids = {
        let mut statement = coordinator.connection().prepare(SQL_OUTBOX_STALE)?;
        let rows = statement.query_map(params![now_utc, OUTBOX_SCAN_LIMIT], |row| {
            row.get::<_, String>(0)
        })?;
        rows.collect::<Result<Vec<_>, _>>()?
    };
    let mut count = 0_u32;
    for id in ids {
        coordinator.connection().execute(
            "update notification_outbox
                set status = 'retry',
                    lease_until = null,
                    lease_token = null,
                    next_attempt_at = ?2
              where outbox_id = ?1 and status = 'leased' and lease_until < ?2",
            params![id, now_utc],
        )?;
        count += 1;
    }
    Ok(count)
}

pub fn claim_due(
    coordinator: &StorageCoordinator,
    now_utc: i64,
    lease_token: &str,
) -> Result<Vec<OutboxIntent>, StorageError> {
    let items = {
        let mut statement = coordinator.connection().prepare(SQL_OUTBOX_DUE)?;
        let rows = statement.query_map(params![now_utc, OUTBOX_SCAN_LIMIT], |row| {
            Ok(OutboxIntent {
                outbox_id: row.get(0)?,
                event_id: row.get(1)?,
                bundle_id: row.get(2)?,
                status: parse_status(&row.get::<_, String>(3)?),
                attempt: row.get(4)?,
                next_attempt_at: row.get(5)?,
                lease_until: row.get(6)?,
                lease_token: row.get(7)?,
                error_class: row.get(8)?,
                error_summary: row.get(9)?,
                idempotency_key: row.get(10)?,
                created_utc: row.get(11)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()?
    };
    let mut claimed = Vec::new();
    let lease_until = now_utc + LEASE_SECS;
    for item in items {
        let changed = coordinator.connection().execute(
            "update notification_outbox
                set status = 'leased',
                    lease_until = ?2,
                    lease_token = ?3
              where outbox_id = ?1
                and status in ('pending', 'retry')
                and next_attempt_at <= ?4
                and (lease_token is null or lease_until is null or lease_until < ?4)",
            params![item.outbox_id, lease_until, lease_token, now_utc],
        )?;
        if changed == 1 {
            let mut next = item;
            next.status = OutboxStatus::Leased;
            next.lease_until = Some(lease_until);
            next.lease_token = Some(lease_token.to_string());
            claimed.push(next);
        }
    }
    Ok(claimed)
}

pub fn complete_send(
    coordinator: &StorageCoordinator,
    item: &OutboxIntent,
    lease_token: &str,
    result: Result<(), NotifyError>,
    now_utc: i64,
) -> Result<OutboxStatus, StorageError> {
    match result {
        Ok(()) => {
            coordinator.connection().execute(
                "update notification_outbox
                    set status = 'sent',
                        lease_until = null,
                        lease_token = null,
                        error_class = null,
                        error_summary = null
                  where outbox_id = ?1 and lease_token = ?2",
                params![item.outbox_id, lease_token],
            )?;
            Ok(OutboxStatus::Sent)
        }
        Err(error) if error.permanent() || item.attempt + 1 >= MAX_ATTEMPTS => {
            coordinator.connection().execute(
                "update notification_outbox
                    set status = 'failed',
                        attempt = attempt + 1,
                        lease_until = null,
                        lease_token = null,
                        error_class = ?3,
                        error_summary = ?4
                  where outbox_id = ?1 and lease_token = ?2",
                params![
                    item.outbox_id,
                    lease_token,
                    error.class(),
                    error.summary_zh()
                ],
            )?;
            Ok(OutboxStatus::Failed)
        }
        Err(error) => {
            let attempt = item.attempt + 1;
            let delay = backoff_secs(attempt, &item.outbox_id);
            coordinator.connection().execute(
                "update notification_outbox
                    set status = 'retry',
                        attempt = ?3,
                        next_attempt_at = ?4,
                        lease_until = null,
                        lease_token = null,
                        error_class = ?5,
                        error_summary = ?6
                  where outbox_id = ?1 and lease_token = ?2",
                params![
                    item.outbox_id,
                    lease_token,
                    attempt,
                    now_utc + delay,
                    error.class(),
                    error.summary_zh()
                ],
            )?;
            Ok(OutboxStatus::Retry)
        }
    }
}

pub fn scan_once<S: NotificationSink>(
    coordinator: &mut StorageCoordinator,
    sink: &mut S,
    now_utc: i64,
    lease_token: &str,
    locale: crate::i18n::UiLocale,
) -> Result<u32, StorageError> {
    let _ = reclaim_stale(coordinator, now_utc)?;
    let claimed = claim_due(coordinator, now_utc, lease_token)?;
    let mut handled = 0_u32;
    for item in claimed {
        let payload = NotifyPayload {
            title_zh: crate::i18n::t(locale, "notify.alert_title").into(),
            body_zh: format!(
                "{} {}",
                crate::i18n::t(locale, "notify.alert_body"),
                redact_id(&item.event_id)
            ),
            event_id: item.event_id.clone(),
            instance_id: None,
            test_only: false,
        };
        let send = sink.send(&payload);
        let _ = complete_send(coordinator, &item, lease_token, send, now_utc)?;
        handled += 1;
    }
    Ok(handled)
}

pub fn backlog(coordinator: &StorageCoordinator) -> Result<u32, StorageError> {
    let value: i64 = coordinator.connection().query_row(
        "select count(*) from notification_outbox where status in ('pending', 'retry', 'leased')",
        [],
        |row| row.get(0),
    )?;
    Ok(value as u32)
}

pub fn load_item(
    coordinator: &StorageCoordinator,
    outbox_id: &str,
) -> Result<Option<OutboxIntent>, StorageError> {
    coordinator
        .connection()
        .query_row(
            "select outbox_id, event_id, bundle_id, status, attempt, next_attempt_at,
                    lease_until, lease_token, error_class, error_summary, idempotency_key, created_utc
               from notification_outbox where outbox_id = ?1",
            [outbox_id],
            |row| {
                Ok(OutboxIntent {
                    outbox_id: row.get(0)?,
                    event_id: row.get(1)?,
                    bundle_id: row.get(2)?,
                    status: parse_status(&row.get::<_, String>(3)?),
                    attempt: row.get(4)?,
                    next_attempt_at: row.get(5)?,
                    lease_until: row.get(6)?,
                    lease_token: row.get(7)?,
                    error_class: row.get(8)?,
                    error_summary: row.get(9)?,
                    idempotency_key: row.get(10)?,
                    created_utc: row.get(11)?,
                })
            },
        )
        .optional()
        .map_err(StorageError::from)
}

pub fn backoff_secs(attempt: i64, outbox_id: &str) -> i64 {
    let exp = 2_i64.saturating_pow(attempt.min(8) as u32);
    let base = exp.min(BACKOFF_CAP_SECS);
    let digest = Sha256::digest(outbox_id.as_bytes());
    let jitter = i64::from(digest[0] % 7);
    (base + jitter).min(BACKOFF_CAP_SECS)
}

fn status_sql(status: OutboxStatus) -> &'static str {
    match status {
        OutboxStatus::Pending => "pending",
        OutboxStatus::Leased => "leased",
        OutboxStatus::Retry => "retry",
        OutboxStatus::Sent => "sent",
        OutboxStatus::Failed => "failed",
        OutboxStatus::Suppressed => "suppressed",
    }
}

fn parse_status(value: &str) -> OutboxStatus {
    match value {
        "leased" => OutboxStatus::Leased,
        "retry" => OutboxStatus::Retry,
        "sent" => OutboxStatus::Sent,
        "failed" => OutboxStatus::Failed,
        "suppressed" => OutboxStatus::Suppressed,
        _ => OutboxStatus::Pending,
    }
}

fn redact_id(value: &str) -> String {
    if value.len() <= 12 {
        return value.to_string();
    }
    format!("{}…", &value[..12])
}

#[cfg(test)]
mod outbox_worker_tests {
    use super::*;
    use crate::c4::notify::{FakeNotificationSink, NotifyError};
    use crate::storage::migrate;
    use tempfile::tempdir;

    fn seed_pending(coordinator: &StorageCoordinator, id: &str, now: i64) {
        persist_intents(
            coordinator,
            &[OutboxIntent {
                outbox_id: id.into(),
                event_id: format!("ev-{id}"),
                bundle_id: "1:1".into(),
                status: OutboxStatus::Pending,
                attempt: 0,
                next_attempt_at: now,
                lease_until: None,
                lease_token: None,
                error_class: None,
                error_summary: None,
                idempotency_key: format!("idemp-{id}"),
                created_utc: now,
            }],
        )
        .expect("seed");
    }

    #[test]
    fn lease_then_sent() {
        let dir = tempdir().expect("dir");
        let mut coordinator =
            StorageCoordinator::open(&dir.path().join("o.sqlite3")).expect("open");
        seed_pending(&coordinator, "o1", 10);
        let mut sink = FakeNotificationSink::default();
        let handled = scan_once(
            &mut coordinator,
            &mut sink,
            10,
            "tok-a",
            crate::i18n::UiLocale::Zh,
        )
        .expect("scan");
        assert_eq!(handled, 1);
        assert_eq!(sink.sent.len(), 1);
        let item = load_item(&coordinator, "o1").expect("load").expect("row");
        assert_eq!(item.status, OutboxStatus::Sent);
        let _ = migrate;
    }

    #[test]
    fn retry_uses_backoff_then_stale_reclaim() {
        let dir = tempdir().expect("dir");
        let coordinator = StorageCoordinator::open(&dir.path().join("o.sqlite3")).expect("open");
        seed_pending(&coordinator, "o2", 10);
        let claimed = claim_due(&coordinator, 10, "tok-b").expect("claim");
        assert_eq!(claimed.len(), 1);
        let status = complete_send(
            &coordinator,
            &claimed[0],
            "tok-b",
            Err(NotifyError::Temporary("busy")),
            10,
        )
        .expect("complete");
        assert_eq!(status, OutboxStatus::Retry);
        let item = load_item(&coordinator, "o2").expect("load").expect("row");
        assert!(item.next_attempt_at > 10);
        coordinator
            .connection()
            .execute(
                "update notification_outbox set status = 'leased', lease_until = 5, lease_token = 'old' where outbox_id = 'o2'",
                [],
            )
            .expect("stale");
        assert_eq!(reclaim_stale(&coordinator, 20).expect("reclaim"), 1);
        let again = load_item(&coordinator, "o2").expect("load").expect("row");
        assert_eq!(again.status, OutboxStatus::Retry);
        assert!(again.lease_token.is_none());
    }

    #[test]
    fn permanent_failure_is_visible() {
        let dir = tempdir().expect("dir");
        let coordinator = StorageCoordinator::open(&dir.path().join("o.sqlite3")).expect("open");
        seed_pending(&coordinator, "o3", 10);
        let claimed = claim_due(&coordinator, 10, "tok-c").expect("claim");
        let status = complete_send(
            &coordinator,
            &claimed[0],
            "tok-c",
            Err(NotifyError::Permanent("disabled")),
            10,
        )
        .expect("complete");
        assert_eq!(status, OutboxStatus::Failed);
        let item = load_item(&coordinator, "o3").expect("load").expect("row");
        assert_eq!(item.status, OutboxStatus::Failed);
        assert_eq!(item.error_class.as_deref(), Some("permanent"));
    }

    #[test]
    fn double_claim_same_tick_is_exclusive() {
        let dir = tempdir().expect("dir");
        let coordinator = StorageCoordinator::open(&dir.path().join("o.sqlite3")).expect("open");
        seed_pending(&coordinator, "o4", 10);
        let first = claim_due(&coordinator, 10, "tok-1").expect("c1");
        let second = claim_due(&coordinator, 10, "tok-2").expect("c2");
        assert_eq!(first.len(), 1);
        assert!(second.is_empty());
    }
}
