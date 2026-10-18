use std::io::Write;

fn main() {
    embed_twitch_client_id();
    tauri_build::build()
}

/// Embeds the Twitch Client ID from `HIGHBAZAR_TWITCH_CLIENT_ID` (set by the
/// release build, never committed) XOR-masked with a per-build random key, so
/// the ID does not appear as plain text in the binary. This is obfuscation, not
/// encryption: Twitch requires the ID in clear text on every API request.
fn embed_twitch_client_id() {
    println!("cargo:rerun-if-env-changed=HIGHBAZAR_TWITCH_CLIENT_ID");
    let id = std::env::var("HIGHBAZAR_TWITCH_CLIENT_ID").unwrap_or_default();
    let id = id.trim();

    // Per-build key from time and process id; good enough for masking.
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0)
        ^ ((std::process::id() as u64) << 32);
    let mut state = seed | 1;
    let key: Vec<u8> = (0..id.len().max(1))
        .map(|_| {
            // xorshift64
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state >> 24) as u8
        })
        .collect();
    let masked: Vec<u8> = id.bytes().zip(&key).map(|(b, k)| b ^ k).collect();

    let out = std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("twitch_client_id.rs");
    let mut f = std::fs::File::create(out).unwrap();
    writeln!(f, "const MASKED_CLIENT_ID: &[u8] = &{masked:?};").unwrap();
    writeln!(f, "const CLIENT_ID_KEY: &[u8] = &{key:?};").unwrap();
}
