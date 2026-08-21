# Add OpenAI authentication routing switches

## Goal

Allow users who deliberately want OpenAI first-party authentication hosts and core traffic to share the same residential exit to opt into that behavior without broadening the default AI-only routing policy or overloading the existing shared-dependencies switch.

## Background and Confirmed Facts

- The current chain is `local host -> current Profile upstream -> residential SOCKS5 -> AI service`; `dialer-proxy` may resolve to one proxy or one proxy group (`clash-verge-ai-residential.js:805-915,935-963`).
- `ROUTE_OPENAI_CORE = true` covers `chatgpt.com`, `oaiusercontent.com`, `api.openai.com`, and five exact `chat.openai.com` family hosts, but not `auth.openai.com`, `auth0.openai.com`, or `oaistatic.com` (`clash-verge-ai-residential.js:216-263`; `tests/regression.test.js:907-957`).
- `ROUTE_OPENAI_SHARED_DEPENDENCIES` is a third-party/shared-infrastructure bundle (WorkOS, Intercom, SendGrid, Stripe, Cloudflare Challenge, Sentry, and Datadog); enabling it does not route the first-party OpenAI authentication hosts (`clash-verge-ai-residential.js:352-373`).
- The repository deliberately keeps shared login hosts on the original Profile by default and requires the narrowest practical match plus negative tests for any new active domain (`docs/routing-scope.md:78-89`).
- `oaistatic.com` is a web asset surface rather than an identity endpoint, so it must not be silently bundled into an authentication-only switch.
- The current ignored local render has OpenAI core enabled, OpenAI shared dependencies disabled, Google authentication disabled, and process fallback disabled. No credential values are recorded in this task.
- Google authentication already has a separate broad shared-identity switch. Changing its local value is a separate product/risk decision, not required to implement OpenAI controls.
- The user selected two independent public controls. Both remain off by default; the ignored current local TOML enables only OpenAI authentication, while OpenAI web assets and Google authentication remain disabled.
- Routing the first-party authentication hosts does not prove that every login redirect, challenge, SSO, or support dependency uses the residential exit. Third-party shared dependencies remain controlled separately and disabled in the current local TOML.

## Requirements

- R1. Add a dedicated OpenAI authentication routing switch whose domain ownership is separate from `ROUTE_OPENAI_SHARED_DEPENDENCIES`.
- R2. The authentication switch must use the narrowest rule shapes: one bounded suffix for `auth.openai.com` (covering the apex and `setup.auth.openai.com`) and one exact rule for `auth0.openai.com`; it must not add `DOMAIN-SUFFIX,openai.com`.
- R3. Treat `oaistatic.com` as an independently controlled OpenAI web-assets scope. It must not become active merely because authentication or shared dependencies are enabled.
- R4. Both new public/example defaults remain `false`, preserving the current AI-only route policy for existing users and fresh renders.
- R5. Wire every new switch through the full TOML contract: public JavaScript constant, `SWITCH_CONFIG_FIELDS`, example TOML, missing-key completion, local renderer output, configuration documentation, and mapping/default consistency tests.
- R6. Managed-rule and DNS-policy ownership must be symmetrical: enabling adds the intended rules/policies; disabling after a prior render removes only script-owned output while preserving unknown user-owned rules.
- R7. Add positive, negative, independence, default, idempotency, and cleanup regression coverage. Explicit negatives include `www.openai.com`, unrelated `openai.com` subdomains, and third-party/shared dependencies when their own switch is off.
- R8. Do not enable process-wide fallback, broaden Google authentication, add generic `openai.com`, or change the residential SOCKS5/airport chain.
- R9. Update routing-scope documentation so opt-in authentication consistency is an explicit user choice rather than a silent change to the default narrow-scope policy.
- R10. After rendering the ignored current local configuration, set `routing.openai_auth = true` and `routing.openai_web_assets = false`; preserve `routing.antigravity_google_auth = false` and all unrelated local values.
- R11. Keep the generated JavaScript platform-neutral so the same rendered `.local.js` can be copied from Windows into Ubuntu Clash Verge Rev. Do not add Windows paths, shell calls, or host-OS branches to the pasteable script.

## Acceptance Criteria

- [x] AC1. With both new switches at their public defaults, generated rules and DNS policy remain unchanged for OpenAI authentication and `oaistatic.com`; all existing regression expectations still pass.
- [x] AC2. With only OpenAI authentication enabled, `auth.openai.com`, any bounded subdomain such as `setup.auth.openai.com`, and exact `auth0.openai.com` use `AI-家宽`; `oaistatic.com` and unrelated `openai.com` hosts do not.
- [x] AC3. With only OpenAI web assets enabled, `oaistatic.com` and its subdomains use `AI-家宽`; authentication hosts remain on the original Profile.
- [x] AC4. Enabling both switches produces no duplicate rules or DNS-policy keys, and a second `main()` execution is idempotent.
- [x] AC5. Turning either switch off removes that switch's previously generated managed rules without removing unknown user-authored rules targeting `AI-家宽`.
- [x] AC6. A partial or legacy local TOML is completed with both new keys set to the example defaults without changing existing values, comments, credentials, or line-ending style.
- [x] AC7. `node --check`, focused renderer/routing tests, secret scan, and `just ci` pass; actual Clash Verge Rev login-flow verification remains explicitly `UNVERIFIED` until sanitized Connections evidence is collected.
- [x] AC8. The ignored current local TOML and generated script contain OpenAI auth enabled, OpenAI web assets disabled, Google auth disabled, and process-wide fallback disabled, verified without printing credentials or committing either local artifact.
- [x] AC9. The renderer and routing suites pass under the repository's Windows/Ubuntu Node matrix contract; actual Ubuntu Clash host execution remains `UNVERIFIED` until the copied script is loaded into Ubuntu and checked with a sanitized Profile/Connections view.

## Out of Scope

- Proving that routing these domains reduces OpenAI verification challenges or prevents account review.
- Broad `openai.com`, analytics, feature-flag, payment, support, or telemetry routing.
- Changing Google/Antigravity authentication behavior unless the user explicitly includes it in this task.
- Pinning both machines to a particular airport leaf node or changing IPRoyal authentication.

## Notes

- This is a complex cross-contract change. Planning must add `design.md` and `implement.md` before activation.
