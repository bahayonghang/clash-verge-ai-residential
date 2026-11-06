# Dual-client runtime validation

This checklist tests whether the existing AI-only routes actually work in each client. It adds no domains or default capture scope. Complete [dual-client setup](configuration.md#dual-client-setup), then validate the installed versions. Sharing Mihomo, executing the script successfully, or opening a webpage does not establish full compatibility.

## Four combinations and result states

| Validation unit | Record separately |
| --- | --- |
| Windows + Clash Verge Rev | Windows build, client/core versions, global and Profile extensions |
| Windows + FlClash | Windows build, client/core versions, ordinary Profile's Script binding |
| Ubuntu + Clash Verge Rev | Distribution, kernel, desktop session, package type, client/core versions, extensions |
| Ubuntu + FlClash | Distribution, kernel, desktop session, package type, client/core versions, Script binding |

Keep a report for each combination. macOS and FlClash Android need later validation; desktop results do not transfer to them. Preserve gaps when a device, permission, working browser, or core version is unavailable.

| State | Meaning |
| --- | --- |
| PASS | Direct evidence meets the expectation for the specified version, mode, and samples |
| FAIL | Observed behavior contradicts the expectation; describe it |
| UNVERIFIED | Not run, insufficient evidence, or environment unavailable |
| NOT_APPLICABLE | Explicitly outside this run's scope, with a reason; not a substitute for missing access |

Label each item as directly observed, user-reported, or untested. Do not declare all four combinations validated until every applicable item passes.

## Separate four evidence layers

| Layer | What to inspect | What it does not prove |
| --- | --- | --- |
| Script output | Rules, singleton residential group, DNS policies, idempotence | GUI adoption or real residential traffic |
| Native core acceptance | Parsing, resources, and startup with the specified core | Correct client overrides, public DNS, or UDP support |
| Final client configuration | Profile preview/generated file, active loading, later settings | A file alone does not prove the running core loaded it |
| Actual requests | Application chains, DNS paths, TUN/IPv6/WebRTC | One sample does not cover other applications, protocols, or switches |

A standalone Mihomo binary can use its `-t` check on a sanitized isolated configuration; keep its TUN off and avoid production ports. FlClash v0.9.2's `FlClashCore` is a host IPC program whose first argument is an IPC address: **do not run `FlClashCore -t/-v/-h` as a substitute**, or take over production IPC. Its configuration validation only unmarshals RawConfig and is not full Mihomo `-t`. Report a different standalone core separately from FlClash's active loading. [Pinned entry source](https://github.com/chen08209/FlClash/blob/7fd417fb9a9a277c6a9339a8ac87f907df7c25f7/core/cmd/core/main.go#L11), [configuration validation](https://github.com/chen08209/FlClash/blob/7fd417fb9a9a277c6a9339a8ac87f907df7c25f7/core/config.go#L66).

## Check final configuration and application samples

Record the script version/commit or digest, current routing mode, Profile source type, attachment point, active switches, selected upstream, DNS/TUN settings, and observation time. Do not upload full configurations or subscriptions. For FlClash, inspect the running desktop user's configuration; an SSH login can belong to a different user. Storage paths depend on platform and installation, so do not assume a fixed directory.

Compare script output with the final client configuration:

- `AI-家宽` contains only `家宽-SOCKS5`; `dialer-proxy` names a valid airport upstream. Also check the selector's current runtime choice.
- Managed private/AI rules and original Profile fallback remain present. Check whether rule editors or extensions prepend or replace rules.
- Exact/suffix DNS policies, `AI-家宽-DNS-REGEX` and its policy reference, non-AI upstream bindings, and node bootstrap survive. Inspect `fallback`, appended `system://`, and replacement of the entire `nameserver-policy` map.
- Check final `find-process-mode`, top-level `ipv6`, `dns.ipv6`, fake-IP, and TUN fields. Script warnings see input values and cannot detect every later override. Process rules cannot be declared validated while process lookup is disabled.

Choose samples from the current [routing scope](routing-scope.md) and enabled switches. Record sanitized request, rule, and full-chain observations separately:

| Category | Example and expectation |
| --- | --- |
| AI exact / suffix / regex | Choose actual application hosts from each enabled catalog; regional Vertex covers regex, ordinary Google domains do not |
| Authentication and shared assets | Compare enabled `auth.openai.com` / `accounts.google.com` with core requests; judge shared assets by their own switches |
| Non-AI negative samples | Marketplace, download, telemetry, maps, or storage hosts stay on the original Profile; account for its own residential rules |
| Default-off features | With indexing off, current managed rules must not send `repo42.cursor.sh` to residential; a matching fixture does not establish service availability |
| LAN and fallback | A known private service remains usable; ordinary non-AI requests follow the original Profile |

Use actual application requests and client Connections/logs. An explicit HTTP/SOCKS proxy probe proves only that entry path, not system applications' TUN behavior. Connectivity after a script error may simply use the original Profile; an exception is not traffic blocking.

## Five DNS classes and private resolution

| Query class | Configuration expectation | Required runtime evidence |
| --- | --- | --- |
| Enabled AI | Residential policy, including enabled regex ruleset | Residential resolver/outbound chain when an upstream query actually occurs |
| Other overseas | DoH bound to the current Profile upstream | Resolution through that airport upstream |
| Domestic | Domestic DoH via DIRECT | Actual resolver and direct path |
| Private/LAN | System resolver | Private resolver on the relevant interface, private name lookup, and service access |
| Node bootstrap | Separate bootstrap without recursive proxy dependency | Actual node-hostname lookup; explain inapplicable lookups when all nodes use IP literals |

The AI residential DNS expectation above follows the generated policies. Extra sites with `residentialDns: false`, such as AnyRouter, intentionally receive residential application routes without residential DNS policies. Do not classify that separation as a failure or widen DNS scope to pass validation.

Record query type, time, cache effects, actual resolver, and outbound evidence; use UNVERIFIED when unobservable. Distinguish fake-IP A/AAAA answers, SOCKS hostname forwarding, and Mihomo's own queries. Prefer a controlled test domain when a fresh lookup is needed; do not clear system caches without authorization. HTTP exit IP, ECS, resolver geography, and generic DNS test sites cannot establish all five paths. Record browser-specific DoH separately from system DNS.

On Windows, read `Get-DnsClientServerAddress`, distinguish actual physical/TUN interfaces and IPv4/IPv6 resolvers, and compare DHCP/static sources in network settings. On Ubuntu, first identify the resolver manager; where applicable, inspect `resolvectl status`, `readlink -f /etc/resolv.conf`, and NetworkManager's `nmcli device show`. Keep these potentially private outputs local. `127.0.0.53` identifies a local stub, not its upstream; trace the relevant link's resolver. If tools or services are absent, record the current manager instead of installing or switching one for validation.

`system` / `system://` does not guarantee private DNS. Public system resolvers, missing DHCP DNS, multiple interfaces, and browser DoH require actual path checks. FlClash's macOS DNS coordinator does not establish Windows/Ubuntu behavior. See the [DNS and leak model](dns-and-leak-model.md).

## TUN, IPv6, WebRTC, and voice

In the currently authorized mode, check real IPv4 and IPv6 application requests, matched routes, and exits separately. Record the TUN stack, routes, and UDP/TCP DNS interception; check `any:53` and `tcp://any:53` separately. Top-level IPv6 and `dns.ipv6` are distinct. If the host restores IPv6 fake-IP, inspect pool overlap and ordering against broad ULA DIRECT rules without automatically copying rejection rules.

A deliberately non-TUN scope can mark TUN NOT_APPLICABLE with a reason, but cannot claim TUN acceptance. WSL/containers or a standalone core do not replace Ubuntu desktop evidence. Do not disable the current proxy, switch modes, disable adapters, or run both clients' TUNs for a comparison.

Use an approved, functioning WebRTC endpoint and browser. Record ICE candidate types (host, srflx, relay), protocols, and actual paths. Zero candidates, timeouts, or a nonworking endpoint cannot pass. Judge srflx/relay against the expected policy and path; their IPs need not literally equal the HTTP exit. Do not install extensions, change browser policies, block all UDP, or enable shared STUN/TURN capture just to pass a test. If voice is needed, separately test call establishment, bidirectional audio, and stability; ICE success does not establish usable calls.

## Persistence and offline recovery {#recovery}

Refresh subscriptions or restart the client only when authorized; otherwise mark persistence UNVERIFIED. Before an operation, record current availability, active client/mode, Profile and script bindings, selected upstream, and original values of every affected setting. Keep private local backups of the original Profile, script, and settings. Confirm that the backup and client are accessible offline. Never publish or commit credentials or subscriptions from backups.

Fill out this recovery card before changes so the user can execute it without an online agent:

1. Stop the validation operation and record the failing step. Do not add further network changes.
2. In the original client, restore the recorded Profile, script binding, and original values of changed settings. For an old script, regenerate from the preserved original Profile instead of applying it over newer output.
3. Reload/restart the original client only if agreed beforehand. Do not switch to DIRECT or another client, or disable networking. If recovery fails, retain the error and use the local backup with the original client's configuration import entry.
4. Recheck original application and LAN availability; record the result. For a read-only run, write “no recovery needed” rather than claiming tested recovery.

After an authorized refresh/restart, recheck bindings, final configuration, positive/negative samples, and DNS paths. Revisit affected checks after changes to the client, core, script, rule sources, subscription group names, GUI overrides, nodes, network, or browser DNS. Do not automatically update rules or schedule network changes.

## Sanitized report template

Copy once per combination. Publish only minimal conclusions; omit real endpoints/IPs, usernames, personal paths, credentials, cookies, subscription URLs, and raw application logs.

```text
Combination / observation time:
OS, build/distribution, kernel, desktop, package type:
Client version / core version (UNVERIFIED if unknown):
Script version, commit/digest / source documents and pinned SHAs:
Mode / Profile source type / script attachment / enabled switches:
Evidence origin: directly observed / user-reported / untested
Generated file timestamp and consistency with active loading:

Item | State | Sanitized evidence | Limitations
Script output; native acceptance; final client configuration/loading | separate rows | |
AI exact/suffix/regex; authentication; non-AI negatives; LAN/fallback | separate rows | |
DNS AI/overseas/domestic/private/bootstrap; browser DNS | separate rows | |
TUN IPv4/IPv6; WebRTC; voice | separate rows | |
Subscription refresh; persistence after restart | separate rows | |

Offline recovery card: backup location/settings before changes kept locally; steps and trigger
Recovery result: unchanged/no recovery needed / executed with evidence / unverified
Remaining gaps, owner, and next recheck triggers:
```

Record `just ci` and `just docs-build` separately; neither replaces runtime evidence.
