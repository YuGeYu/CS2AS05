# Reproducible build

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

Copy the built files into `src-tauri/resources/inventory-simulator` and update
`manifest.json` plus `SHA256SUMS.txt`. Never label an upstream release hash as
the hash of this downstream build.
