# PlayerSkinMod fixed build record

- Date: 2026-08-11 (Asia/Shanghai)
- Source: `kaecho/CS2-Skin-Forge@75f52fbd5fd0616dbbdd09a65c3a1981593400d1`
- Source project: `upstream/addons/counterstrikesharp/plugins/PlayerSkinMod/PlayerSkinMod.csproj`
- Downstream patch: `patches/player-skin-mod-v1.8.2-version.patch` (two version strings only)
- Target framework: `net8.0`
- CounterStrikeSharp.API package: `1.0.313`
- Build SDK: `.NET SDK 10.0.301`; installed net8 runtime: `8.0.25`
- Commands: `dotnet restore PlayerSkinMod.csproj`; `dotnet build PlayerSkinMod.csproj -c Release --no-restore`
- Result: 0 warnings, 0 errors
- Bundle plugin version: `1.8.2`
- `PlayerSkinMod.dll`: 76,288 bytes; SHA-256 `0DAC2B47275EC6AD308F8F5F4797140D1AA8BA63DA93F2840DA90BA9A3838388`
- `PlayerSkinMod.json`: 413 bytes; SHA-256 `D9D686289899663C92795C04738B563E065DF8B7170DF02B8518F2C666A19706`
- `skins_en.json`: 340,305 bytes; SHA-256 `DFD0A2CB407065FC0B567E67891FC7E2DFA3AB1C794D1708FF1DDBCBAD29F98B`
- Bundle resource: `src-tauri/resources/skin-forge/PlayerSkinMod/`

Static inspection confirms the 200 ms `FileSystemWatcher` debounce, `Server.NextFrame` handoff, `GiveNamedItem` hook, weapon/knife/glove refresh, stickers, keychains, agents, music kits, random mode, and the `skin_menu`, `skin_random`, and `skin_reset` commands.

The local target previously reported CounterStrikeSharp `1.0.371`, newer than compile-time API `1.0.313`. The assistant does not replace that framework. Actual binary compatibility remains pending the user's offline game log.
