# Optional source build for investigation

The application ships the official `InventorySimulator-v3.3.0.zip` Release
asset directly. The following command is for source investigation only; its
output is not copied into application resources.

From this directory:

```powershell
dotnet build .\upstream\InventorySimulator.csproj -c Release --nologo
```

The plugin output is written to:

```text
upstream/bin/Release/plugins/InventorySimulator/
```

The gamedata output is written to:

```text
upstream/bin/Release/gamedata/inventory-simulator.json
```

For a future update, extract the pinned official Release asset into
`src-tauri/resources/inventory-simulator` and regenerate `manifest.json` plus
`SHA256SUMS.txt`. Never label a local build as the official Release asset.
