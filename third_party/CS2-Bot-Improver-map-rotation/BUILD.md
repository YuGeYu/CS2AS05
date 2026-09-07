# MapRotation build

```powershell
dotnet build .\addons\counterstrikesharp\plugins\MapRotation\MapRotation.csproj -c Release --nologo
```

The release output is copied into the main package by the controlled packaging
script. Do not claim the package is updated until the ZIP manifest and SHA-256
checks pass.
