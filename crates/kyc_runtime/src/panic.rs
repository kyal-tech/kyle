// kyc_runtime::panic — Panic handler

pub fn ky_panic(message: &str) -> ! {
    eprintln!("KL PANIC: {}", message);
    std::process::abort();
}
