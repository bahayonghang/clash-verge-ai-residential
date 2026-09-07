# Branch governance validation (2026-09-07)

Repo: `bahayonghang/clash-verge-ai-residential`  
Branch: `main`  
Owner type: User (no organization rulesets)  
Write: classic branch-protection PUT only. No `git push`. No history rewrite. No force-push.

This note is the five-tool shared governance evidence for
`.trellis/spec/frontend/quality-guidelines.md` Main Branch Protection.
The spec file records the GET. The spec file is not the remote setting.

## Before GET

Source: `research/protection-before.json` (scratch copy
`c5-protection-before.json`). Redacted: no auth headers, no tokens.
The protection JSON contains no credentials.

| Field | Value |
|---|---|
| `required_status_checks.strict` | `true` |
| `required_status_checks.checks` | `[{ "context": "Required checks", "app_id": 15368 }]` |
| `required_status_checks.contexts` (GET-derived) | `["Required checks"]` |
| `required_approving_review_count` | `0` |
| `dismiss_stale_reviews` | `false` |
| `require_code_owner_reviews` | `false` |
| `require_last_push_approval` | `false` |
| `enforce_admins.enabled` | `true` |
| `required_conversation_resolution.enabled` | `true` |
| `required_linear_history.enabled` | `false` |
| `allow_force_pushes.enabled` | `false` |
| `allow_deletions.enabled` | `false` |
| `block_creations.enabled` | `false` |
| `lock_branch.enabled` | `false` |
| `allow_fork_syncing.enabled` | `false` |
| `required_signatures.enabled` | `false` |
| `restrictions` | absent on GET |

## Rulesets and merge methods

Read before PUT and again after PUT.

- `GET /repos/bahayonghang/clash-verge-ai-residential/rulesets` → `[]`
- `GET /repos/bahayonghang/clash-verge-ai-residential/rulesets?includes_parents=true` → `[]`
- `GET /repos/bahayonghang/clash-verge-ai-residential/rules/branches/main` → `[]`

No repository ruleset affects merge on `main`. Classic branch protection is the
merge gate.

Repo merge settings (unchanged by this write):

- `allow_squash_merge`: `true`
- `allow_rebase_merge`: `true`
- `allow_merge_commit`: `true`

Squash and rebase remain available after linear history is required.

## PUT

Converted GET → PUT schema. Did not send the GET body as-is.

- `required_status_checks`: `strict` + app-bound `checks` only. Omitted `contexts`.
- `enforce_admins`: boolean from GET `.enabled`
- `required_pull_request_reviews`: current counts/flags
- `restrictions`: `null` (GET had no restriction object)
- `required_linear_history`: `true` (only intentional change)
- `allow_force_pushes`, `allow_deletions`, `block_creations`,
  `required_conversation_resolution`, `lock_branch`, `allow_fork_syncing`:
  booleans from GET `.enabled`
- Did not send `required_signatures` (separate endpoint). GET had signatures
  disabled; left disabled.

PUT body (scratch `c5-protection-put.json`):

```json
{
  "required_status_checks": {
    "strict": true,
    "checks": [
      {"context": "Required checks", "app_id": 15368}
    ]
  },
  "enforce_admins": true,
  "required_pull_request_reviews": {
    "dismiss_stale_reviews": false,
    "require_code_owner_reviews": false,
    "required_approving_review_count": 0,
    "require_last_push_approval": false
  },
  "restrictions": null,
  "required_linear_history": true,
  "allow_force_pushes": false,
  "allow_deletions": false,
  "block_creations": false,
  "required_conversation_resolution": true,
  "lock_branch": false,
  "allow_fork_syncing": false
}
```

`PUT /repos/bahayonghang/clash-verge-ai-residential/branches/main/protection`
returned HTTP success (`gh api` exit 0).

## After GET

Source: `research/protection-after.json` (scratch copy
`c5-protection-after.json`). Independent GET immediately after PUT.

Field diff versus before snapshot:

| Field | Before | After |
|---|---|---|
| `required_linear_history.enabled` | `false` | `true` |

Every other comparable protection field equals the before snapshot, including
strict app-bound `Required checks`, PR review flags, `enforce_admins`,
conversation resolution, force-push, deletion, `block_creations`,
`lock_branch`, `allow_fork_syncing`, and `required_signatures.enabled=false`.
GET still derives `contexts: ["Required checks"]` from `checks`. That is the
documented GET shape, not a PUT mix of `contexts` and `checks`.

## Hosted PR evidence

UNVERIFIED. This change did not open a test PR. The next authorized ordinary
PR must supply exact-head `Required checks` evidence. Administrator bypass
must not be used for that proof.

## Check-agent independent GET (2026-09-07)

Check-agent `GET /repos/bahayonghang/clash-verge-ai-residential/branches/main/protection`
matched `research/protection-after.json`, scratch `c5-protection-after.json`,
and scratch `c5-protection-live-verify.json`. Saved as
`research/protection-check.json`.

Comparable field diff versus `research/protection-before.json`:

| Field | Before | Check GET |
|---|---|---|
| `required_linear_history.enabled` | `false` | `true` |

No other comparable protection field differed.

Repo merge methods (repo GET, not the protection PUT):

- `allow_squash_merge`: `true`
- `allow_rebase_merge`: `true`
- `allow_merge_commit`: `true`

Rulesets remain empty:

- `GET .../rulesets` → `[]`
- `GET .../rulesets?includes_parents=true` → `[]`
- `GET .../rules/branches/main` → `[]`

GitHub `main` SHA is `caa580650d6d1bcb5b338f5e1c9ccd4afff3e7ea`. The latest
`PushEvent` on `refs/heads/main` is 2026-09-03T03:27:04Z,
`ee6618d9caab8980f7d6de686ebf9941a9ef6831` → `caa58065` (PR merge). No force-push
and no history rewrite in this task. This task did not `git push`.

## Not done

- No force-push
- No history rewrite
- No `git push`
- No change to merge-method flags
- No change to `required_signatures`
- No ruleset write
- No C1/C2/C3/C4 product-file edit
