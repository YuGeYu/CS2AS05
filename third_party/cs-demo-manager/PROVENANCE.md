# akiver/cs-demo-manager provenance

- Repository: https://github.com/akiver/cs-demo-manager
- Commit: `8961f5072fe4d42803dde68e8e71b3c90b216504`
- Version observed at retrieval: `3.20.1`
- Retrieved: 2026-07-30
- License: MIT, preserved in `LICENSE`
- Source archive SHA-256: `A48BFD7D14FC1267213516DA6388F9D857FACA251A18598C2D0BEBC8AE7449AA`

Reused assets and source facts:

- `static/images/maps/cs2/radars/*.png` -> `src/assets/maps/cs2/radars/*.png` (byte-for-byte)
- `static/images/maps/cs2/thumbnails/*.png` -> `src/assets/maps/cs2/thumbnails/*.png` (byte-for-byte)
- `src/node/database/maps/default-maps.ts` -> `src/data/cs2-map-metadata.ts` (mechanical CS2-only data conversion)
- `src/ui/maps/get-scaled-coordinate-x.ts` and `get-scaled-coordinate-y.ts` -> coordinate helpers in `src/data/cs2-map-metadata.ts` (TypeScript rewrite with equivalent formulas)

No Electron, PostgreSQL, account integration, analyzer sidecar, or application code is embedded. The asset mapping, byte lengths, and downstream SHA-256 values are recorded in `map-manifest.json`.
