# LaihoE/demoparser provenance

- Repository: https://github.com/LaihoE/demoparser
- Commit: `ba39cc44cd5abfd7f34df2b3c0a7dd3630048311`
- Retrieved: 2026-07-28
- Original directories: `src/parser`, `src/csgoproto`
- License: MIT, preserved in `LICENSE`

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
