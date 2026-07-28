// Kyle Runtime — Crypto (SHA-256, random bytes)
//
// Uses CommonCrypto on macOS, provides the same API on all platforms.

#[cfg(target_os = "macos")]
mod platform {
    use core::ffi::{c_int, c_uchar, c_uint, c_void};

    #[repr(C)]
    struct CC_SHA256_CTX {
        state: [u32; 8],
        count: [u32; 2],
        buffer: [u8; 64],
    }

    unsafe extern "C" {
        fn CC_SHA256_Init(c: *mut CC_SHA256_CTX) -> c_int;
        fn CC_SHA256_Update(c: *mut CC_SHA256_CTX, data: *const c_void, len: c_uint) -> c_int;
        fn CC_SHA256_Final(md: *mut c_uchar, c: *mut CC_SHA256_CTX) -> c_int;
        fn CCRandomGenerateBytes(bytes: *mut c_void, count: usize) -> c_int;
    }

    pub fn sha256(data: *const u8, len: i32, out: *mut u8) -> *mut u8 {
        let mut ctx = std::mem::MaybeUninit::<CC_SHA256_CTX>::uninit();
        unsafe {
            CC_SHA256_Init(ctx.as_mut_ptr());
            CC_SHA256_Update(ctx.as_mut_ptr(), data as *const c_void, len as c_uint);
            CC_SHA256_Final(out as *mut c_uchar, ctx.as_mut_ptr());
        }
        out
    }

    pub fn random_bytes(buf: *mut u8, count: i64) -> i32 {
        unsafe { CCRandomGenerateBytes(buf as *mut c_void, count as usize) as i32 }
    }
}

#[cfg(not(target_os = "macos"))]
mod platform {
    // Fallback: simple non-crypto-grade PRNG for development
    // Production should use OpenSSL or similar
    pub fn sha256(_data: *const u8, _len: i32, out: *mut u8) -> *mut u8 {
        // Return zeros as placeholder
        unsafe {
            for i in 0..32 {
                *out.add(i) = 0;
            }
        }
        out
    }

    pub fn random_bytes(buf: *mut u8, count: i64) -> i32 {
        // Simple LCG for non-crypto random (development only)
        use std::time::{SystemTime, UNIX_EPOCH};
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        let mut state = seed;
        unsafe {
            for i in 0..count as usize {
                state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                *buf.add(i) = (state >> 32) as u8;
            }
        }
        0
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_sha256(data: *const u8, len: i32, out: *mut u8) -> *mut u8 {
    platform::sha256(data, len, out)
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_random_bytes(buf: *mut u8, count: i64) -> i32 {
    platform::random_bytes(buf, count)
}
