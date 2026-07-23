# Troubleshooting

## No usable upstream was found

Symptoms include an exception mentioning `dialer-proxy`, Profile candidates, or `MATCH/FINAL`.

Check:

- the actual top-level group name in the generated Profile;
- `PROFILE_UPSTREAM_OVERRIDES` spelling, spaces, and emoji;
- whether the Profile has a final `MATCH` or `FINAL` rule;
- whether the candidate is `DIRECT`, `REJECT`, `AI-家宽`, or `家宽-SOCKS5`.

## Reserved-name collision

The names `AI-家宽` and `家宽-SOCKS5` are managed by the script. Rename any unrelated Profile proxy or group that already uses either name.

## Recursive proxy-group error

The selected upstream graph eventually references itself, `AI-家宽`, or `家宽-SOCKS5`. Remove the reference from the subscription override or select a clean top-level airport group. The script also adds exclusion filters to `include-all` groups, but it cannot safely repair every arbitrary group graph.

## Placeholder credential error

The public template keeps `server`, `username`, and `password` as `xxx`. Either:

- edit a private `*.local.js` copy; or
- predefine an existing `家宽-SOCKS5` node in the Profile so the script can reuse its endpoint and credentials.

For no-auth SOCKS5, both credential fields must be empty strings.

## AI service works but assets do not

This can be expected under AI-only routing. Marketplace, update, download, media, analytics, and shared dependencies use the original Profile route. Inspect the failed host before widening scope. Prefer one exact domain over a broad provider suffix.

## Cursor Marketplace or YouTube hits AI-家宽

The active script should not inject those rules. Common causes:

- stale v5.3 rules remain in another script or merge layer;
- a broad user rule such as `DOMAIN-SUFFIX,cursor.com,AI-家宽` exists outside this script;
- process-wide fallback was manually enabled;
- the running Profile was not refreshed after editing the global script.

Search the generated configuration for the matching rule and identify which enhancement layer supplied it.

## DNS leak test does not show the residential location

This is expected for generic test domains. v5.4 sends only AI-domain DNS through `AI-家宽`; ordinary overseas DNS uses the current airport upstream. Validate an AI hostname in Mihomo DNS logs or connection metadata instead.

## Chat/voice or realtime feature fails

The default AI-only policy does not capture generic STUN/TURN or all realtime UDP ports. Confirm the exact product host and the UDP capability of both the airport path and residential SOCKS5 service before enabling shared realtime switches.
