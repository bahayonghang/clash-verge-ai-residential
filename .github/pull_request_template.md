## Change summary

Describe the routing, DNS, compatibility, test, or documentation change.

## Evidence

- [ ] Every new domain is tied to an official service document or a sanitized Clash/Mihomo connection record.
- [ ] Shared infrastructure is not routed through `AI-家宽` without a documented reason.
- [ ] No proxy address, credential, subscription URL, generated profile, or unredacted log is included.

## Routing boundary

- Newly routed hosts:
- Explicitly excluded hosts:
- Expected impact on non-AI traffic:

## Validation

- [ ] `npm run ci`
- [ ] Repeated execution remains idempotent.
- [ ] Cursor Marketplace/download/update traffic remains outside `AI-家宽`.
- [ ] YouTube, Maps, advertising, shared telemetry, and public DNS remain outside `AI-家宽`.
- [ ] At least one real Profile was tested with secrets removed from all logs.

## Rollback

Describe how to disable or revert the change safely.
