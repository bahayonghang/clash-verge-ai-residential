# Multi-Profile Upstream Resolution

Mihomo's `dialer-proxy` field is a single name. The script resolves one valid name at runtime for the current Clash Verge Rev Profile.

## Resolution order

1. Candidates in `PROFILE_UPSTREAM_OVERRIDES[profileName]`.
2. `HOME_PROXY_TEMPLATE["dialer-proxy"]`, normally `🚀节点选择`.
3. Names in `UPSTREAM_CANDIDATES`.
4. The target of the last `MATCH` or `FINAL` rule when `ALLOW_FINAL_RULE_UPSTREAM_FALLBACK` is enabled.
5. Optional semantic-name guessing when `ALLOW_HEURISTIC_UPSTREAM_FALLBACK` is enabled. It is disabled by default.

The first existing and structurally valid proxy/group is selected. Resolution never writes an array to `dialer-proxy` and never silently falls back to `DIRECT`.

Spaces and emoji are valid in an upstream name. `#` and `&` are not: Mihomo uses them as delimiters in the upstream-bound DoH URL, so the script rejects a selected name containing either character before building DNS configuration.

## Recursion protection

Before injecting the residential node, the script:

- excludes `家宽-SOCKS5` from every `include-all` or `include-all-proxies` group;
- removes script-managed group references from the selected upstream graph;
- rejects direct and indirect group cycles;
- rejects reserved-name collisions;
- rejects top-level upstreams that explicitly disable UDP;
- rejects an upstream that resolves to `DIRECT`, `REJECT`, the residential node, or the `AI-家宽` group.

## Runtime limitation

Static configuration can prove that a selector exists, but it cannot reliably read the selector's current runtime choice. Do not put `DIRECT`, `REJECT`, `家宽-SOCKS5`, or `AI-家宽` inside the selector used as `dialer-proxy`.

## Diagnostics

The script logs one line after successful transformation:

```text
[AI-家宽 v5.5.0] Profile“<name>”：dialer-proxy -> <resolved group>
```

When resolution fails, use the sanitized proxy-group names and the final `MATCH`/`FINAL` rule to update the candidate list. Do not publish node endpoints or provider URLs.
