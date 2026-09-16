# Upstream provenance

- Repository: <https://github.com/ianlucas/cs2-css-inventory-simulator>
- Release: `v3.1.1`
- Tag commit: `6d2cb8a99e021b88523b77079b4a4ce765b44807`
- License: MIT (`LICENSE` and `upstream/License.txt`)
- Imported: 2026-09-16
- Target framework: `.NET 8.0`
- CounterStrikeSharp API: `1.0.371`

The `upstream/` directory is a source snapshot from the fixed commit above. The
assistant carries one downstream behavior patch: `invsim_ws_enabled` defaults
to `true`, so the documented `!ws` refresh flow works without a transient
server-console command. All inventory modeling and CS2 runtime behavior remain
upstream implementations.

Do not update this directory from a floating branch. A future update must pin a
new tag and commit, preserve its license, rebuild from source, regenerate every
SHA-256 value, and pass real local `-insecure` CS2 acceptance.
