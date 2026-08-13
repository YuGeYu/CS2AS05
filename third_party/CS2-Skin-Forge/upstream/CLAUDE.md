# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**CS2-Skin-Forge** is a Counter-Strike 2 skin customization mod with two main components:
- **Panel** (Frontend): A Tauri + React desktop application for configuring custom skins, knives, gloves, agents, and music kits
- **Addon** (Backend): A C# plugin using CounterStrikeSharp that applies the customizations in-game

The panel and addon communicate via a JSON file (`player_loadout.json`) written by the panel and read by the plugin on player spawn.

### Architecture

```
CS2-Skin-Forge/
├── Panel/                                    # Tauri + React desktop app
│   ├── src/                                  # React frontend (TypeScript + Tailwind CSS)
│   │   ├── components/                       # UI tabs and panels
│   │   ├── data/                             # Static skin/knife/glove/agent data
│   │   ├── i18n/                             # Localization (EN/ZH/JA/KO/RU)
│   │   ├── lib/                              # Tauri API calls
│   │   ├── utils/                            # Type definitions
│   │   └── styles/                           # Global CSS and Tailwind
│   ├── src-tauri/src/                        # Rust backend for file handling
│   │   ├── lib.rs                            # File I/O, path resolution, CS2 plugin deployment
│   │   └── main.rs                           # Entry point
│   └── package.json                          # Frontend dependencies
├── addons/counterstrikesharp/plugins/PlayerSkinMod/
│   ├── PlayerSkinModPlugin.cs                # Main plugin, event hooks
│   ├── Services/
│   │   ├── WeaponService.cs                  # Skin/glove/knife application logic
│   │   └── LoadoutService.cs                 # Loadout file parsing and reloading
│   ├── Models/PlayerLoadout.cs               # Data model for player_loadout.json
│   ├── Data/StaticData.cs                    # DefIndex mappings for skins, knives, gloves
│   └── PlayerSkinMod.csproj                  # C# project file
└── .github/workflows/build.yml               # CI/CD: builds Panel for Windows/Linux/macOS and C# plugin
```

## Common Development Commands

### Building the Frontend (React + Tauri)

```bash
cd Panel

# Install dependencies
npm install

# Dev server with hot reload (Vite + Tauri)
npm run dev

# Build for release (all platforms: Windows, Linux, macOS)
npm run tauri build

# Build for specific platform (optional)
npm run tauri build -- --target x86_64-pc-windows-msvc    # Windows
npm run tauri build -- --target x86_64-unknown-linux-gnu  # Linux
npm run tauri build -- --target aarch64-apple-darwin      # macOS
```

### Building the C# Plugin

```bash
# Windows (requires .NET 8.0 SDK)
dotnet build addons/counterstrikesharp/plugins/PlayerSkinMod/PlayerSkinMod.csproj -c Release

# Linux/macOS: create placeholder DLL (plugin only runs on Windows in CS2)
touch addons/counterstrikesharp/plugins/PlayerSkinMod/PlayerSkinMod.dll
```

### Running Tests

There are currently no automated tests. Manual testing is performed:
1. Run the panel in dev mode: `npm run dev` (from `Panel/`)
2. Test UI changes and file I/O
3. Deploy addon to local CS2 installation and verify in-game

### Type Checking

```bash
cd Panel

# TypeScript type check
npm run build  # includes tsc type-check before Vite build
```

### Linting

There is no automated linter configured yet. Code style follows:
- **Frontend**: React/TypeScript conventions, Tailwind CSS utilities
- **Backend Rust**: Cargo.toml defaults
- **Plugin C#**: Net8.0 with nullable reference types enabled

## Key Technologies

- **Frontend**: React 18, TypeScript, Vite, Tailwind CSS, Tauri 2
- **Backend (Rust)**: Tauri 2 runtime, file I/O with `std::fs`, JSON parsing with `serde_json`
- **Plugin**: C# (Net8.0), CounterStrikeSharp 1.0.313
- **Build**: Cargo (Rust), npm/Node.js 18+, dotnet (C#)
- **CI/CD**: GitHub Actions
- **Localization**: 6 languages (English, Simplified Chinese, Traditional Chinese, Japanese, Korean, Russian)

## Important Architecture Patterns

### Data Flow: Panel → Plugin

1. **User configures loadout in React panel** (weapons, knives, gloves, agents, music kits, stickers, keychains)
2. **Panel writes to `player_loadout.json`** via Rust backend (using Tauri API, finds CS2 path in settings)
3. **Plugin watches file** for changes and reloads on spawn
4. **Plugin applies skins** by modifying FallbackPaintKit/FallbackSeed/FallbackWear or changing models

### Skin Application in Plugin

The plugin hooks two key events:
- **`GiveNamedItem`**: Most weapons. Applied when weapon is given to player.
- **`OnEntitySpawned`**: Fallback for weapons that bypass GiveNamedItem.

For knives, gloves, and agents:
- **Knives**: `ChangeSubclass` to swap the knife model
- **Gloves**: Applied on player spawn via `OnEntitySpawned`
- **Agents**: Replaced via entity index manipulation on spawn

### Localization (i18n)

Frontend supports 6 languages via `src/i18n/` directory:
- **English** (`english`)
- **Simplified Chinese** (`schinese`)
- **Traditional Chinese** (`tchinese`)
- **Japanese** (`japanese`)
- **Korean** (`koreana`)
- **Russian** (`russian`)

The system auto-detects language from OS/browser locale (e.g., `zh-CN` → simplified, `zh-TW` → traditional). Each language has:
- UI strings (buttons, labels) in `dictionary.ts`
- Localized names for agents, music kits (from `data/localNames.ts`)

Language state is a module-level store in `src/i18n/index.ts` consumed via `useSyncExternalStore` — calling `setLanguage()` re-renders every component using `useT()` immediately. English keys in `keys.ts` are the source of truth; every other locale is a partial override that falls back to English. When adding UI strings, always add the key to `keys.ts` AND translations to all 5 locale blocks in `dictionary.ts` — never hardcode display strings in components.

The plugin has English-only names in C#, but the panel translates on the frontend.

### Static Data

- **Skins & Weapons**: `Panel/src/data/skins.ts`, `weapons.ts`, `skinNamesEn.ts`
- **Knives**: `Panel/src/data/knives.ts`, `knifeSkins.ts`
- **Agents**: `Panel/src/data/localNames.ts` (localized names only; DefIndex mappings in plugin)
- **Music Kits**: `Panel/src/data/localNames.ts` (localized names)
- **Gloves**: DefIndex/paint mappings in plugin's `StaticData.cs`
- **Stickers**: `Panel/src/data/stickers.ts` (cosmetic weapon stickers, 4 slots per weapon)
- **Keychains/Charms**: `Panel/src/data/keychains.ts` (decorative weapon attachments)

## Key Files to Know

### Frontend (Panel/src/)

| File | Purpose |
|------|---------|
| `App.tsx` | Main component, tab routing, loadout state management |
| `components/TabNavigation.tsx` | Tab switcher (Weapons, Knives, Gloves, Agents, Music, Settings) |
| `components/SettingsPanel.tsx` | CS2 path, language selection, deploy addon |
| `components/WeaponPanel.tsx` | Weapon grid; clicking a weapon opens `WeaponEditorModal` |
| `components/KnifePanel.tsx` | Knife type grid per team; clicking opens `KnifeEditorModal` |
| `components/GlovePanel.tsx` | Glove type grid per team; clicking opens `GloveEditorModal` |
| `components/AgentPanel.tsx` | Agent model selection |
| `components/MusicKitPanel.tsx` | Music kit selection |
| `components/ui/Modal.tsx` | Shared modal shell (ESC/backdrop close, title bar, footer) — all dialogs build on it |
| `components/ui/PickerGrid.tsx` | Searchable paginated item grid used by all pickers (skins, stickers, keychains) |
| `components/editors/WeaponEditorModal.tsx` | Full-size weapon editor: tabs for skin / stickers / keychain / details (nametag + StatTrak) |
| `components/editors/KnifeEditorModal.tsx` | Full-size knife skin editor (per team, wear/seed) |
| `components/editors/GloveEditorModal.tsx` | Full-size glove paint editor (per team, wear/seed) |
| `lib/api.ts` | Tauri invocations (file I/O, CS2 path resolution, addon deployment) |
| `data/` | Static mappings: weapons, skins, knives, agents, stickers, keychains |

### Backend (Panel/src-tauri/src/)

| File | Purpose |
|------|---------|
| `lib.rs` | File I/O: read/write `player_loadout.json`, resolve CS2 path, deploy addon files |

Note: `save_loadout`/`load_loadout` pass the loadout through as untyped `serde_json::Value` on purpose — the Rust layer only persists the file. The data model lives in `Panel/src/utils/types.ts` (writer) and `Models/PlayerLoadout.cs` (reader); do NOT reintroduce a typed Rust mirror of the loadout.

### Plugin (addons/counterstrikesharp/plugins/PlayerSkinMod/)

| File | Purpose |
|------|---------|
| `PlayerSkinModPlugin.cs` | Main plugin class, game event hooks, spawn handling |
| `Services/WeaponService.cs` | Apply skins to weapons, knives, gloves on spawn |
| `Services/LoadoutService.cs` | Parse loadout JSON, watch for file changes, trigger reload |
| `Data/StaticData.cs` | Hardcoded DefIndex mappings for skins, knives, gloves, agents |
| `Models/PlayerLoadout.cs` | Data model for `player_loadout.json` |

## Development Workflow

### Adding a New Weapon Skin

1. Add skin data to `Panel/src/data/skins.ts` (DefIndex, paint kit ID, etc.)
2. Frontend automatically displays it in the weapon picker
3. On spawn, plugin reads the loadout and applies via `WeaponService.ApplySkin()`

### Adding a New Knife

1. Add to `Panel/src/data/knives.ts` and `knifeSkins.ts`
2. Add DefIndex mapping to `Data/StaticData.cs` in the plugin
3. Frontend shows in knife selector; plugin applies via `ChangeSubclass()`

### Adding Localized Agent Names

1. Add entry to `Panel/src/data/localNames.ts` for each language
2. Frontend displays on agent tab
3. Plugin stores DefIndex only in loadout JSON (no localization needed server-side)

### Adding Weapon Cosmetics (Stickers, Keychains)

1. **Stickers** (5 slots per weapon):
   - Add to `Panel/src/data/stickers.ts` with image URL
   - `WeaponEditorModal.tsx` (Stickers tab) handles selection UI automatically
   - Stored in `player_loadout.json` as `weaponStickers: { [defindex]: [{id, offsetX, offsetY, wear, scale, rotation}, ...] }`
   - Plugin applies via CounterStrikeSharp's sticker API

2. **Keychains** (1 per weapon):
   - Add to `Panel/src/data/keychains.ts` with name, image, and localization
   - `WeaponEditorModal.tsx` (Keychain tab) handles selection UI automatically
   - Stored in `player_loadout.json` as `weaponKeychains: { [defindex]: {id, offsetX, offsetY, offsetZ, seed} }`
   - Plugin applies via keychain entity spawning

### Testing Panel Changes

```bash
cd Panel
npm run dev
# App opens in dev window with hot reload
# Test file writing by checking that player_loadout.json updates in the CS2 directory
# Test localization by changing language in Settings
# Test cosmetic selection and display in preview
```

Key testing areas when modifying the panel:
- **File I/O**: Verify `player_loadout.json` is created/updated correctly at the CS2 path
- **Localization**: Check that all UI strings and localized names display correctly in each language
- **State Management**: Ensure loadout state persists correctly across tab switches
- **Data Validation**: Verify selected DefIndex values match the plugin's `StaticData.cs` mappings

### Testing Plugin Changes

1. Build plugin: `dotnet build addons/counterstrikesharp/plugins/PlayerSkinMod/PlayerSkinMod.csproj -c Release`
2. Copy `PlayerSkinMod.dll` to CS2's addon directory
3. Reload CS2 or restart server
4. Spawn in-game and verify skin is applied

### In-Game Console Commands

The plugin exposes the following console commands (accessible in-game via the console):
- `skin_menu`: Reload the loadout from the panel (use after changing skins in the panel without respawning)
- `skin_random`: Enable random skin mode (randomizes skins on each spawn)
- `skin_reset`: Reset all skins to defaults

## Common Gotchas

### Music Kit IDs Are Real IDs, Not Indexes

`loadout.musicKit` stores the actual in-game MusicKitID (e.g. 3 = Crimson Assault), matching `Panel/src/data/skins.ts` `musicKits[].id`. The plugin's `StaticData.KitIds` array is only a pool for random selection — never index into it with `loadout.MusicKit` (that bug shipped once and played the wrong kits).

### Language Code Mapping

Frontend uses these internal codes that differ from standard locale codes:
- `english` (not `en`)
- `schinese` (Simplified Chinese, not `zh-CN`)
- `tchinese` (Traditional Chinese, not `zh-TW`)
- `japanese` (not `ja`)
- `koreana` (not `ko`)
- `russian` (not `ru`)

These are defined in `src/i18n/index.ts`. Auto-detection maps standard locale codes (e.g., `zh-TW`) to these internal codes.

### Plugin Only Builds on Windows

The plugin is a C# .NET 8.0 project that compiles to `PlayerSkinMod.dll`. Only Windows has the .NET SDK in CI. On Linux/macOS, a placeholder DLL is created (plugin won't run, but CI still succeeds).

### Glove Paint Kit Mismatch

Each glove type (Bloodhound, Sport, Driver, etc.) has its own 3D model with different UV layouts. Using a paint kit intended for one glove on another causes texture distortion. The panel auto-resets glove paint when the type changes; the plugin does not validate on its end.

### File Encoding

The panel writes `player_loadout.json` via Rust's `std::fs` (UTF-8). The plugin reads it via `serde_json`. Ensure no non-UTF-8 characters are in the loadout (player names, custom strings, etc.).

### Hot Reload in Dev

`npm run dev` enables Vite hot reload for frontend changes. Rust backend changes (in `src-tauri/src/`) require a full rebuild. For rapid iteration, use the dev server and reload the Tauri window manually if needed.

## Release Process

Releases are automated via GitHub Actions:

1. Update `RELEASE_NOTES.md` at the repo root — the release job publishes it verbatim as the GitHub Release body (`body_path` in `build.yml`)
2. Push a tag: `git tag v1.8.0 && git push --tags`
3. CI builds Panel and plugin for Windows, Linux, macOS
4. Artifacts are uploaded and a GitHub Release is created with the notes from `RELEASE_NOTES.md`
5. Files are automatically versioned in the release (e.g., `CS2-Skin-Mod_1.8.0_x64-setup.exe`)

The version must be updated in these places before tagging (keep them identical):
- `Panel/package.json` → `version`
- `Panel/src-tauri/Cargo.toml` → `version` (then run `cargo check` to refresh `Cargo.lock`)
- `Panel/src-tauri/tauri.conf.json` → `version`
- `addons/counterstrikesharp/plugins/PlayerSkinMod/PlayerSkinMod.json` → `Version`
- `addons/counterstrikesharp/plugins/PlayerSkinMod/PlayerSkinModPlugin.cs` → `ModuleVersion`

The UI reads the version at runtime via `getVersion()` (from `tauri.conf.json`) — never hardcode version strings in components.

## Debugging Tips

- **Frontend**: Open DevTools in Tauri window (Ctrl+Shift+I / Cmd+Shift+I) to inspect React state and console errors
- **File I/O**: Check that CS2 path is correctly resolved by inspecting `player_loadout.json` location
- **Plugin Errors**: Check CS2 server console (if hosting locally) for CounterStrikeSharp logs
- **Skin Not Applied**: Verify DefIndex and paint kit ID are correct in `StaticData.cs`; check that plugin reloads on file change in `LoadoutService.cs`
