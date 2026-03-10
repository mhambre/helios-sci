#[allow(dead_code)]
pub(crate) mod functions;
pub(crate) mod numbers;

// Helios shares the same syscall ABI as Linux
cfg_if::cfg_if! {
    if #[cfg(any(
        all(target_arch = "x86_64", target_os = "helios"),
        all(target_arch = "x86_64", target_os = "linux")
    ))] {
        #[allow(dead_code)]
        pub(crate) mod syscall;
    } else {
        compile_error!("Unsupported Target OS");
    }
}
