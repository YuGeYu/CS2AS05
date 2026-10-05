# Upstream provenance

- Repository: <https://github.com/ianlucas/cs2-css-inventory-simulator>
- Release: `3.5.0`
- Tag commit: `157086c73718920285d3d761fe22c47a98090ec1`
- License: MIT (`LICENSE` and `upstream/License.txt`)
- Imported: 2026-10-05
- Target framework: `.NET 10.0`
- CounterStrikeSharp API: `1.0.375`
- Source archive SHA-256: `A6F024A42D46ECB9267943B4C3E4E0F042D215BCED1DE408428B9F93877D6B74`
- Upstream release ZIP SHA-256: `9119DA8EFA655156DDB65D0F79F395A4DC1EEB158D3F357545797A8133FB8315`

The `upstream/` directory is an unmodified source snapshot from the fixed commit
above. The application bundles the official `InventorySimulator-v3.5.0.zip`
release asset directly, without a downstream plugin build or behavior patch.
The upstream `invsim_ws_enabled` default is `false`. v3.5.0 adds pet spawning,
warmup-only pet respawn, and free-roam controls without changing that default.

Do not update this directory from a floating branch. A future update must pin a
new tag and commit, preserve its license, import the fixed release asset,
regenerate every SHA-256 value, and pass real local `-insecure` CS2 acceptance.
