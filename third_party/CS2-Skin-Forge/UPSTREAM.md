# CS2-Skin-Forge upstream record

- Public migration repository: https://github.com/kaecho/CS2-Skin-Forge
- Original user-supplied repository: https://github.com/emptysuns/CS2-Skin-Forge (anonymous access returned 404 on 2026-08-11).
- Tag: `v1.8.2` (annotated tag object `51d1d2955a975abd954cb024eda224a2489b12a6`).
- Peeled commit: `75f52fbd5fd0616dbbdd09a65c3a1981593400d1`.
- Previous fixed baseline: `b2edea17db9128609dd41f726f179cd965206433`.
- Ancestry: `git merge-base --is-ancestor` returned 0; the new commit is three commits ahead.
- Acquisition: detached Git checkout under `%TEMP%`, copied at 2026-08-11 (Asia/Shanghai).
- Vendor root: `third_party/CS2-Skin-Forge/upstream/`; `.git`, `bin`, `obj`, and `node_modules` are excluded.
- Full source manifest: `SHA256SUMS.txt`.

The fixed commit declares `GPL-3.0` in `README.md`, `README_CN.md`, and `README_RU.md`, but does not contain a standalone `LICENSE` file. The README files and this missing-license-file condition are preserved; no license text has been invented.

The upstream release is internally versioned inconsistently: the repository tag and Panel Rust crate are `1.8.2`, while `PlayerSkinModPlugin.cs`, `PlayerSkinMod.json`, Panel `tauri.conf.json`, and release notes remain `1.8.1`. Vendor files remain unchanged. The bundle applies `patches/player-skin-mod-v1.8.2-version.patch`, which changes only the plugin module and manifest version strings to `1.8.2`; this downstream distinction must remain visible in release reports.

The Tauri application deploys only the fixed, hashed build artifact recorded in `BUILD.md`. It never synthesizes a placeholder DLL and never overwrites CounterStrikeSharp.
