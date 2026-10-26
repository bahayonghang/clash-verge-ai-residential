# Implementation plan

Do this only after the planning summary is approved. Edit the public template and its tests/docs. Do not touch `*.local.toml`, `*.local.js`, or the running FlClash profile.

## Checklist

1. In `clash-verge-ai-residential.js`, append the four hosts to `GROK_EXACT_DOMAINS` and extend the Chinese comment with the sign-in doc and the wildcard-certificate limit. Do not add suffix constants.
2. Update `tests/regression.test.js`:
   - `GROK_EXACT_DOMAINS` deep equality.
   - Positive route and bare DNS keys for the four hosts.
   - Negative route and absent `+.<parent>` / parent apex DNS keys, including `slack.hooks.x.ai`.
   - `withPatchedSwitches({ ROUTE_GROK_CORE: false })` drops the four hosts.
   - The existing `grok_web_assets=false` case still matches them.
   - Add the four `DOMAIN` lines to the managed-rule cleanup fixture.
   - Add them to the v5.10 positive host list.
   - Change the default AI-家宽 rule count from 55 to 59.
3. Update Chinese and English copies together:
   - `README.md`
   - `docs/configuration.md`, `docs/local-configuration.md`, `docs/routing-scope.md`
   - `docs/en/configuration.md`, `docs/en/local-configuration.md`, `docs/en/routing-scope.md`
   - `CHANGELOG.md` under `[未发布]`
   State the four exact hosts, that `grok_core` owns them, and that parent domains stay out.
4. Run `npm test`. Then run `just ci` because this is a routing-list change. Do not treat `npm run ci` as a substitute for `just ci`.

## Rollback point

The only behavioral edit is `GROK_EXACT_DOMAINS` plus the tests that lock it. Revert that constant to `["auth.x.ai"]` to restore the previous route set.

## Review gate

`trellis-check` after the edit. Spec focus: `.trellis/spec/frontend/quality-guidelines.md` (official evidence, negative scope, bare exact DNS keys, `just ci`). Research: `research/grok-bot-live-hosts.md`.
