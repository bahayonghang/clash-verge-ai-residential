# Troubleshooting

## No usable upstream was found

Symptoms include an exception mentioning `dialer-proxy`, Profile candidates, or `MATCH/FINAL`.

Check:

- the actual top-level group name in the generated Profile;
- `PROFILE_UPSTREAM_OVERRIDES` spelling, spaces, and emoji;
- whether the selected upstream name contains `#` or `&`; these delimit Mihomo's DoH URL fragment and are rejected rather than encoded;
- whether the Profile has a final `MATCH` or `FINAL` rule;
- whether the candidate is `DIRECT`, `REJECT`, `AI-家宽`, or `家宽-SOCKS5`.

## Reserved-name collision

The names `AI-家宽` and `家宽-SOCKS5` are managed by the script. Rename any unrelated Profile proxy or group that already uses either name.

## Recursive proxy-group error

The selected upstream graph eventually references itself, `AI-家宽`, or `家宽-SOCKS5`. Remove the reference from the subscription override or select a clean top-level airport group. The script also adds exclusion filters to `include-all` groups, but it cannot safely repair every arbitrary group graph.

## Placeholder credential error

The public template keeps `server`, `username`, and `password` as `xxx`. Either:

- edit the ignored `clash-verge-ai-residential.local.toml` and regenerate with `just render-local` or `node scripts/sync-local-config.js`; or
- predefine an existing `家宽-SOCKS5` node in the Profile so the script can reuse its endpoint and credentials.

For no-auth SOCKS5, both credential fields must be empty strings. Never hand-edit the generated `.local.js`; change the TOML and render again.

## AI service works but assets do not

This can be expected under AI-only routing. Marketplace, update, download, media, analytics, and shared dependencies use the original Profile route. Inspect the failed host before widening scope. Prefer one exact domain over a broad provider suffix.

## Cursor Marketplace or YouTube hits AI-家宽

Cursor routing is disabled by default in v5.5, and Marketplace or YouTube is excluded even when Cursor core is enabled. Common causes of an unexpected match are:

- stale rules remain in a subscription, another script, or Global Extend Config (Merge);
- a broad user rule such as `DOMAIN-SUFFIX,cursor.com,AI-家宽` exists outside this script;
- process-wide fallback was manually enabled;
- the running Profile was not refreshed after editing the global script.

v5.5 replaces only rules that the current version can generate. It deliberately preserves unknown rules and no longer migrates pre-v5.4 output. If v5.4 output was manually persisted, the following retired Cursor rules are also user-owned and require manual removal:

```text
DOMAIN,repo42.cursor.sh,AI-家宽
DOMAIN-REGEX,^[a-z0-9-]+\.api5\.cursor\.sh$,AI-家宽
DOMAIN-REGEX,^(?:us-asia|us-eu|us-only)\.gcpp\.cursor\.sh$,AI-家宽
```

Also search for broader old or custom entries such as:

```text
DOMAIN-SUFFIX,cursor.com,AI-家宽
DOMAIN,www.youtube.com,AI-家宽
DOMAIN,marketplace.cursorapi.com,AI-家宽
```

Identify which enhancement layer supplied each match, remove stale entries from that source, then refresh the Profile. Do not add the retired strings to the current script merely to clean them up.

## DNS leak test does not show the residential location

This is expected for generic test domains. The script sends only AI-domain DNS through `AI-家宽`; ordinary overseas DNS uses the current airport upstream. Validate an AI hostname in Mihomo DNS logs or connection metadata instead.

## The first connection to a new non-AI domain is slower

Strict DNS rebuilding sends real non-AI overseas lookups through DoH bound to the current Profile upstream. When a GEOIP fallback needs a real lookup, the first query for a new domain can add roughly one airport round trip; cache hits do not pay the same setup cost. This trade-off is retained to keep resolver routing consistent and resistant to pollution. See [DNS and Leak Model](dns-and-leak-model.md).

## Chat/voice or realtime feature fails

The default AI-only policy does not capture generic STUN/TURN or all realtime UDP ports. Confirm the exact product host and the UDP capability of both the airport path and residential SOCKS5 service before enabling shared realtime switches.
