#![cfg(target_os = "ios")]

use talpid_types::net::wireguard::PrivateKey;

/// Generate a new random WireGuard private key, writing 32 bytes to `key_out`.
/// This function is safe to call concurrently with different pointers. Not safe to call
/// concurrently with the same pointers.
///
/// # Safety
/// `key_out` must be a valid pointer to a 32-byte buffer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mullvad_generate_private_key(key_out: *mut u8) {
    let key = PrivateKey::new_from_random();
    let bytes = key.to_bytes();
    // SAFETY: `key_out` must point to a 32-byte buffer as documented above.
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), key_out, 32);
    }
}

/// Derive a WireGuard public key from a private key.
/// This function is safe to call concurrently if different parameters are used.
///
/// # Safety
/// `private_key` must be a valid pointer to 32 bytes.
/// `public_key_out` must be a valid pointer to a 32-byte buffer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mullvad_derive_public_key(
    private_key: *const u8,
    public_key_out: *mut u8,
) {
    // SAFETY: `private_key` must point to 32 bytes as documented above.
    let private_bytes: [u8; 32] = unsafe { std::ptr::read(private_key as *const [u8; 32]) };
    let key = PrivateKey::from(private_bytes);
    let public_key = key.public_key();
    let public_bytes = public_key.as_bytes();
    // SAFETY: `public_key_out` must point to a 32-byte buffer as documented above.
    unsafe {
        std::ptr::copy_nonoverlapping(public_bytes.as_ptr(), public_key_out, 32);
    }
}
