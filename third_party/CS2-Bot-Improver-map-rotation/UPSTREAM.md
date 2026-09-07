# MapRotation upstream provenance

- Repository: https://github.com/YuGeYu/CS2-Bot-Improver
- Branch: `main`
- Fixed commit: `f1b0cc450ecb5d3479def51cb2c4f3e0e98f60bb`
- Source path: `addons/counterstrikesharp/plugins/MapRotation/MapRotation.cs`
- Upstream project file: `addons/counterstrikesharp/plugins/MapRotation/MapRotation.csproj`
- Upstream author/module: LBTV / MapRotation 1.0.0
- Upstream license: GPL-3.0 (repository README)

The downstream source contains one product-default patch: `_enabled` is initialized
to `true` on plugin load so the assistant's local-BOT installation actually performs
automatic rotation. The `lbtv_map_rotation 0|1` command remains available to pause
or resume rotation. The downstream project file changes the target framework and
CounterStrikeSharp API package to match this assistant's net10.0 / 1.0.371 plugin
toolchain. The compiled DLL is redistributed inside the main `CS2BotImprover.zip`
package as `MapRotation`.

Downstream package entry:

- `addons/counterstrikesharp/plugins/MapRotation/MapRotation.dll`
- DLL SHA-256: `F6C2C8713200B754B1142C1DD862DACBCC753B721E488BA62623732F4ABCEC96`
- Main package SHA-256 after integration and manifest regeneration: `7E58C5E4739E9B2BDE99F274C5DC0CDA4D9142B1BD7CA8E672956A59DA3683A4`
