#[allow(dead_code)]
pub(crate) mod errno;
pub(crate) mod functions;

cfg_if::cfg_if! {
    if #[cfg(any(
        all(target_arch = "x86", target_os = "helios"),
        all(target_arch = "x86", target_os = "linux")
    ))] {
        #[allow(dead_code)]
        pub(crate) mod syscall;
    } else {
        compile_error!("Unsupported target OS for util module");
    }
}
