/// Get the sha256 hash, aka the function discriminator, of a function name. For a function that is
/// in lib.rs, pass "global", "functionname", e.g. get_function_hash("global", "somefunc")
/// 
/// For functions with no args, no other data is required, e.g. you can pass this slice.to_vec()
/// 
/// If the function has args, serialize them and append them to this buff.
pub fn get_function_hash(namespace: &str, name: &str) -> [u8; 8] {
    let preimage = format!("{}:{}", namespace, name);
    let mut sighash = [0u8; 8];
    sighash.copy_from_slice(
        &anchor_lang::solana_program::hash::hash(preimage.as_bytes()).to_bytes()
            [..8],
    );
    sighash
}