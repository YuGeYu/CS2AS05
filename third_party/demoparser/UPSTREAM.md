# LaihoE/demoparser provenance

- Repository: https://github.com/LaihoE/demoparser
- Commit: `ba39cc44cd5abfd7f34df2b3c0a7dd3630048311`
- Retrieved: 2026-07-28
- Original directories: `src/parser`, `src/csgoproto`
- License: MIT, preserved in `LICENSE`
- Source archive SHA-256: `AA5B2722E675289F700ACB401BFC536753BE6EA06A78ECECB812DD2F95C234C0`

Local build patch:

- `csgoproto/build.rs` returns early when the checked-in generated
  `src/protobuf.rs` exists. This prevents release builds from cloning a moving
  GameTracking-CS2 checkout and requiring protoc. Parser behavior is unchanged.

CS2AS 0.5.5 local adapter patches:

- `first_pass/stringtables.rs` keeps non-HLTV userinfo entries with a non-empty
  name even when `xuid == 0`; BOT identity is keyed by userid instead of a fake
  shared SteamID.
- `second_pass/game_events.rs` always exposes the raw userid and falls back to
  userinfo name/SteamID when an entity mapping is unavailable.
- `PlayerEndMetaData` exposes `user_id`, `controller_id`, and `is_bot`; the
  final parser merge retains BOT rows instead of filtering SteamID zero.
- Old end-of-match messages can encode BOT slots as small non-zero `xuid`
  values. Only values in the SteamID64 individual-account range are exposed as
  Steam IDs; smaller values remain BOT identities keyed by userid/controller.
- `second_pass/collect_data.rs` treats requested events and player properties
  as independent outputs, so a single parse can return both the timeline and
  final controller totals when no explicit tick filter is supplied.
- The application adapter requests entity parsing and controller totals first,
  and retries only known `IllegalPathOp` failures with userinfo/event recovery.

Coverage: the three local CS2 samples listed in
`docs/version-0.5.5-startup-default-demo-scoreboard-fix-plan-20260728.md`;
the 63.8 MB current sample is the strict-path baseline and the two older
samples exercise the fallback path. Unknown parser errors are still surfaced.

Reproducibility check (2026-07-30): after normalizing CRLF/LF, all vendored
files match the fixed archive except the following documented downstream patch
surface. `SOURCE-MANIFEST.sha256` hashes the exact downstream bytes and excludes
ignored build output and local Demo fixtures.

- `csgoproto/build.rs`
- `parser/README.md`
- `parser/src/e2e_test.rs`
- `parser/src/parse_demo.rs`
- `parser/src/first_pass/stringtables.rs`
- `parser/src/second_pass/collect_data.rs`
- `parser/src/second_pass/game_events.rs`
- `parser/src/second_pass/other_netmessages.rs`
- `parser/src/second_pass/parser_settings.rs`
- `parser/src/second_pass/variants.rs`
