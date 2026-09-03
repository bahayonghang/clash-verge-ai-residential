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

From v5.6, Cursor core routing is on by default. From v5.9.0, repository indexing hosts `repo[0-9]+.cursor.sh` moved to `routing.cursor_repository_indexing`, default `false`, falling back to the original Profile / airport upstream. A missing local TOML field is completed as `false`; set `true` to restore v5.8.1 repo residential routing. Even with Cursor core routing on, Marketplace and YouTube stay out of scope. To send Cursor core traffic through the airport upstream as well, set `routing.cursor_core = false` in the local TOML.

If `repo42.cursor.sh` still hits `AI-家宽`, check whether `routing.cursor_repository_indexing` is `true`, and whether the subscription or Merge layer still has a user-owned `DOMAIN,repo42.cursor.sh,AI-家宽`. Privacy Mode does not stop indexing uploads. With `disableHttp2` or a server-forced HTTP/1.1 fallback, RepositoryService may move to shared `api2.cursor.sh`; that host stays under `cursor_core`, and Clash domain rules cannot isolate that fallback while keeping most Cursor APIs. Default-off indexing cannot claim all repository uploads are excluded.

When the Clash Verge script console only shows `Script execution failed`, read `%APPDATA%\io.github.clash-verge-rev.clash-verge-rev\logs\latest.log`. `Script execution error: expected value at line 1 column 1` means `main` threw and returned empty. The usual cause is pasting the public template `clash-verge-ai-residential.js` (`HOME_PROXY_TEMPLATE` is `xxx`) into Global Extend Script while the current Profile has no preset `家宽-SOCKS5` node. Paste `clash-verge-ai-residential.local.js` from `just render-local`.

Common causes of unexpected hits:

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

Strict DNS rebuilding sends real non-AI overseas lookups through DoH bound to the current Profile upstream. When a GEOIP fallback needs a real lookup, the first query for a new domain can add roughly one airport round trip; cache hits do not pay the same setup cost. This trade-off is retained to keep resolver routing consistent and resistant to pollution. See [DNS and leak model](dns-and-leak-model.md).

## Chat/voice or realtime feature fails

The default AI-only policy does not capture generic STUN/TURN or all realtime UDP ports. Confirm the exact product host and the UDP capability of both the airport path and residential SOCKS5 service before enabling shared realtime switches.

## Fresh offline install fails configuration validation (`geosite.dat`)

The rebuilt DNS policy includes `geosite:cn` and `geosite:private`, both of which need `geosite.dat`. Mihomo downloads that file on first use. If the device is offline and has no copy, configuration parsing fails and Clash Verge Rev reports a validation error. On first launch the app falls back to a minimal default config. Connect once so Mihomo can fetch the geo database (most subscriptions trigger the same download), or place a valid `geosite.dat` in the Mihomo working directory and refresh the Profile.

## Script TUN DNS hijack and IPv6 settings do not apply

Current Clash Verge Rev restores control-plane fields (`tun`, `ipv6`, mode, ports) from app settings after the global script runs. Script-completed TUN DNS hijack and `ipv6: false` therefore do not take effect on these hosts; that logic exists only for older hosts. Configure the IPv6 switch and TUN DNS hijack on the Clash Verge Rev settings page. DNS servers, `nameserver-policy`, and fake-ip rebuilt by the script are unaffected; if Clash Verge Rev DNS override is enabled, `dns.ipv6` is also restored from app settings.

## Warning that references were removed from an upstream group

The upstream graph reachable from the resolved `dialer-proxy` must not contain `AI-家宽` or `家宽-SOCKS5`, or the chain recurses. When the script finds such a reference it removes it and logs the group name and removed entries. Route AI traffic with rules whose target is `AI-家宽`; do not nest that group inside the upstream selector. If you need a custom AI selector, keep it out of the `家宽-SOCKS5` upstream graph.

## Private CIDR rules override custom LAN routing

The script inserts loopback and RFC1918 direct rules before user rules, because those rules must sit ahead of every process-fallback rule. If the Profile intentionally forwards `10.0.0.0/8` or similar private ranges through an enterprise proxy group, the injected `DIRECT` rules match first. That is a fail-closed trade-off; adjust routing expectations for the affected ranges, or disable the script for that Profile.
