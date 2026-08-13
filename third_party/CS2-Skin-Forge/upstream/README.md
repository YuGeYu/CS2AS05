# CS2 Skin Mod

A local-only skin customization plugin for Counter-Strike 2 that allows players to customize weapon skins, knives, gloves, agent models, and music kits.

**[中文说明](README_CN.md)**
**[Описание на русском](README_RU.md)**

## Demo

The following screenshots are from local bot matches using the [CS2-Bot-Improver](https://github.com/ed0ard/CS2-Bot-Improver) plugin:

> CS2-Bot-Improver is a plugin for Counter-Strike 2 that improves bots' aim, movement, nade throwing, personalities, strategies, etc. Aims to enhance your experience when playing against bots offline or with friends. It can be installed on both clients and servers.

![Demo 1](imgs/demo1.png)
![Demo 2](imgs/demo2.png)
![Demo 3](imgs/demo3.png)

[Click to view more demo screenshots](./screenshots.md)

## VAC Risk Warning

**IMPORTANT: This plugin modifies CS2 game files and requires the `-insecure` launch option.**

While this plugin:
- Only modifies local client-side rendering
- Does not affect real Steam inventory
- Is designed for offline/local play only

**Using any third-party software with CS2 carries inherent risk.** Valve's anti-cheat system (VAC) may flag modified game files. The authors are **NOT responsible** for any VAC bans or account restrictions that may occur.

**Use at your own risk.**

> **Safety Tip:** For maximum safety, use a secondary Steam account when playing locally with this plugin.

## Features

- **Weapon Skins**: Customize skins for 30+ weapons
- **Knives**: Choose from 20 knife types with multiple skin options
- **Gloves**: 8 glove types with multiple skin variants, per-team configuration (CT/T separate gloves), localized names
- **Agent Models**: 35 CT + 44 T agent models (localized names: EN/ZH/JA/KO)
- **Music Kits**: 18+ music kits with localized names
- **Keychains/Charms**: Attach weapon charms with custom position and seed
- **Nametags**: Set custom weapon names via nametags
- **StatTrak**: Enable StatTrak kill counters with custom kill counts
- **Random Mode**: Randomize all skins on each spawn
- **Custom Mode**: Mix and match specific skins
- **Desktop Panel**: Native Tauri application for easy customization
- **Multi-language**: English, Simplified Chinese, Traditional Chinese, Japanese, Korean, Russian

## Installation

### Prerequisites

1. Counter-Strike 2 installed
2. CounterStrikeSharp — the panel will automatically detect and install it for you when you click "Deploy Addons". No manual installation required.

### Step 1: Download and install the panel

Download the latest release for your platform:
- Windows: `CS2-Skin-Mod_x.x.x_x64-setup.exe`
- Linux: `cs2-skin-mod-panel_x.x.x_amd64.AppImage`
- macOS: `CS2-Skin-Mod_x.x.x_aarch64.dmg`

Install and launch the application.

### Step 2: Deploy addons

1. Launch the CS2 Skin Mod Panel
2. Go to **Settings** (gear icon in top-right)
3. Set your CS2 path (e.g. `C:\Program Files (x86)\Steam\steamapps\common\Counter-Strike Global Offensive\game\csgo`)
4. Click **"Deploy Addons"** -- this automatically installs CounterStrikeSharp (if missing) and copies the plugin files to your CS2 directory

### Step 3: Configure CS2 Launch Options

1. Open Steam
2. Right-click on Counter-Strike 2
3. Select "Properties"
4. In "Launch Options", add: `-insecure`

**Warning: This will prevent you from playing on VAC-secured servers.**

### Step 4: Customize and play

1. Customize your loadout in the panel
2. Click **"Apply Loadout"** to save
3. Launch CS2 with `-insecure` and start a local match
4. Respawn in-game to see your skins

## Usage

### Quick Start

1. Launch the CS2 Skin Mod Panel
2. Go to **Settings** and set your CS2 path
3. Click **"Deploy Addons"** to install the plugin
4. Customize your loadout in the panel
5. Click **"Apply Loadout"** to save
6. Launch CS2 with `-insecure` and start a local match
7. Respawn in-game to see your skins

### In-Game Commands

| Command | Description |
|---------|-------------|
| `skin_menu` | Reload loadout from panel (use after changing skins in panel) |
| `skin_random` | Enable random skin mode |
| `skin_reset` | Reset all skins to default |

### Enhanced Bot Experience (Recommended)

For the best experience with bot matches, we recommend using [CS2-Bot-Improver](https://github.com/ed0ard/CS2-Bot-Improver) alongside this plugin.

> **Tip:** We recommend installing CS2-Bot-Improver first — it also deploys the CounterStrikeSharp framework automatically. Then install our skin plugin — it will detect the existing CounterStrikeSharp and only deploy its own mod files.

### How It Works

The panel and addon communicate via a JSON file (`player_loadout.json`):
1. The **Panel** writes your loadout configuration to the plugin directory
2. The **Addon** reads this file when players spawn and applies the skins
3. The addon watches for file changes and auto-reloads when you save from the panel

The plugin uses CounterStrikeSharp to hook into CS2's weapon system:
- Intercepts `GiveNamedItem` calls to apply skins when weapons are given
- Intercepts `OnEntitySpawned` to catch weapons that bypass the GiveNamedItem hook
- Modifies `FallbackPaintKit`, `FallbackSeed`, and `FallbackWear` attributes
- Changes knife models via `ChangeSubclass` input
- Replaces agent models and gloves on player spawn

All changes are **client-side only** and do not affect other players or the server.

### Language Support

The panel supports multiple languages. Click the Settings icon to change between:
- English
- Simplified Chinese
- Traditional Chinese
- Japanese
- Korean
- Russian

## Building from Source

### Prerequisites

- Node.js 18+
- Rust toolchain
- Tauri CLI

### Build Steps

```bash
# Clone the repository
git clone https://github.com/kaecho/CS2-Skin-Forge.git
cd CS2-Skin-Forge

# Install frontend dependencies
cd Panel
npm install

# Build the Tauri app
npm run tauri build
```

## Troubleshooting

### Skins not showing

1. Verify `-insecure` is in launch options
2. Ensure CounterStrikeSharp is installed (click "Deploy Addons" in Settings to auto-install it)
3. Make sure you set the correct CS2 path in Settings
4. Click "Deploy Addons" to ensure the plugin is installed
5. Respawn after applying changes
6. Check server console for errors

### Glove textures misaligned or broken

This is usually caused by a mismatch between the glove type (DefIndex) and the paint kit. Each glove type (Bloodhound, Sport, Driver, etc.) uses a different 3D model with its own UV layout -- applying a paint kit intended for one glove type to another will produce distorted textures.

**Fix:** When you switch glove types in the panel, the paint is automatically reset to a valid default for that glove type. If you still see issues:
1. Select your desired glove type first
2. Then pick a paint from the list (the panel only shows valid paints for that type)
3. Click "Apply Loadout" and respawn in-game
4. If textures are still broken, set the glove to "Random Mode", apply, respawn, then re-select your desired glove

### Panel not saving loadout

1. Make sure the CS2 path is set correctly in Settings
2. Check that the `addons/counterstrikesharp/plugins/PlayerSkinMod/` directory exists
3. Try clicking "Apply Loadout" again

### Game crashes

1. Verify game files integrity
2. Remove conflicting plugins
3. Check CS2 console for errors

## File Structure

```
CS2-Skin-Forge/
├── addons/
│   └── counterstrikesharp/
│       └── plugins/
│           └── PlayerSkinMod/
│               ├── PlayerSkinModPlugin.cs  # Main plugin
│               ├── PlayerSkinMod.csproj    # C# project
│               ├── Services/
│               │   ├── WeaponService.cs    # Skin/glove/knife application
│               │   └── LoadoutService.cs   # Loadout file parsing
│               ├── Data/
│               │   └── StaticData.cs       # Static skin/knife/glove data
│               ├── Models/
│               │   └── PlayerLoadout.cs    # Loadout data model
│               ├── skins_en.json           # Skin data (EN names + legacy flags)
│               └── player_loadout.json     # Loadout (generated by panel)
├── Panel/
│   ├── src/                                # React frontend (with i18n)
│   ├── src-tauri/                          # Tauri backend
│   └── package.json
├── .github/
│   └── workflows/
│       └── build.yml                       # CI/CD config
├── README.md                               # This file
└── README_CN.md                            # Chinese documentation
```

## Credits

- Based on [CS2-Bot-Improver](https://github.com/ed0ard/CS2-Bot-Improver) by ed0ard
- Glove skin application references [Nereziel/cs2-WeaponPaints](https://github.com/Nereziel/cs2-WeaponPaints)
- Uses [CounterStrikeSharp](https://github.com/roflmuffin/CounterStrikeSharp)
- Built with [Tauri](https://tauri.app/) + [React](https://react.dev/)

## License

GPL-3.0

## Disclaimer

This software is provided for **educational purposes only**. Use at your own risk. The authors are not responsible for:
- VAC bans or account restrictions
- Game crashes or instability
- Any other consequences of using this software

By using this software, you acknowledge that you understand the risks involved.
