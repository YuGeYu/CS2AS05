# Upstream provenance

- Repository: <https://github.com/ianlucas/cs2-css-inventory-simulator>
- Release: `3.3.0`
- Tag commit: not included in the supplied release archive
- License: MIT (`LICENSE` and `upstream/License.txt`)
- Imported: 2026-09-25
- Target framework: `.NET 10.0`
- CounterStrikeSharp API: `1.0.375`
- Source archive SHA-256: `233CD51A0E887F97DD7336BE181D2C9842338FFC24C88DC33F74E8153CD76915`
- Upstream release ZIP SHA-256: `AED5379AFE5E1E24FDB099A8D8AD427FEDF1014A312D7167653BA384C1313AAC`

The `upstream/` directory is an unmodified source snapshot from the fixed commit
above. The application bundles the official `InventorySimulator-v3.3.0.zip`
release asset directly, without a downstream plugin build or behavior patch.
The upstream `invsim_ws_enabled` default is `false`.

Do not update this directory from a floating branch. A future update must pin a
new tag and commit, preserve its license, import the fixed release asset,
regenerate every SHA-256 value, and pass real local `-insecure` CS2 acceptance.
