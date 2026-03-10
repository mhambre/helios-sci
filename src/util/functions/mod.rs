cfg_if::cfg_if! {
    if #[cfg(all(target_arch = "x86_64", target_os = "linux"))] {
        mod linux;
        #[allow(dead_code)]
        pub(crate) use linux::*;
    } else {
        compile_error!("Unsupported Target OS");
    }
}
