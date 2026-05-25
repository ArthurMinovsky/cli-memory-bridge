use std::iter;

use crate::provider::ProviderKind;

const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

pub fn derive_resume_hash(provider: ProviderKind, conversation_id: &str) -> String {
    let mut hash = FNV_OFFSET_BASIS;

    for byte in provider
        .as_slug()
        .bytes()
        .chain(iter::once(b':'))
        .chain(conversation_id.bytes())
    {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }

    format!("{hash:016x}")
}
