use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-env-changed=CS2AS_DEFAULT_API_KEY");

    let key = env::var("CS2AS_DEFAULT_API_KEY").unwrap_or_default();
    let encoded = key
        .bytes()
        .enumerate()
        .map(|(index, byte)| byte ^ key_mask(index))
        .collect::<Vec<_>>();
    let generated = format!("const DEFAULT_AI_KEY_OBFUSCATED: &[u8] = &{:?};\n", encoded);
    let output = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR is set by Cargo"))
        .join("default_ai_key.rs");
    if fs::read_to_string(&output).ok().as_deref() != Some(generated.as_str()) {
        fs::write(output, generated).expect("write generated AI connection data");
    }

    tauri_build::build()
}

fn key_mask(index: usize) -> u8 {
    0xA7u8.wrapping_add((index as u8).wrapping_mul(31))
}
