fn main() {
    // Ensure PlayerSkinMod.dll exists so include_bytes! compiles.
    // In CI the real DLL is built by dotnet before cargo; this is a no-op there.
    let dll = "../../addons/counterstrikesharp/plugins/PlayerSkinMod/PlayerSkinMod.dll";
    if !std::path::Path::new(dll).exists() {
        std::fs::write(dll, []).ok();
    }
    tauri_build::build()
}
