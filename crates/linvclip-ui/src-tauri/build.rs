fn main() {
    // ── Embed obfuscated KLIPY API key at compile time ──
    // Reads from klipy.key (gitignored), XOR-scrambles the bytes so the key
    // never appears as plaintext in the binary or in version control.
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let key_path = std::path::Path::new(&manifest_dir).join("klipy.key");
    let raw_key = if key_path.exists() {
        std::fs::read_to_string(&key_path)
            .unwrap_or_default()
            .trim()
            .to_string()
    } else {
        String::new()
    };

    let xor_pad: &[u8] = b"LvCb2026xKm9";
    let obfuscated: Vec<u8> = raw_key
        .bytes()
        .enumerate()
        .map(|(i, b)| b ^ xor_pad[i % xor_pad.len()])
        .collect();

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest = std::path::Path::new(&out_dir).join("klipy_key.rs");
    std::fs::write(
        &dest,
        format!(
            "const KLIPY_KEY_XOR_PAD: &[u8] = b\"LvCb2026xKm9\";\nconst KLIPY_KEY_BYTES: &[u8] = &{:?};\n",
            obfuscated
        ),
    )
    .expect("Failed to write klipy_key.rs");

    tauri_build::build()
}
