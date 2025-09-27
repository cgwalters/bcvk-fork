#[cfg(target_os = "linux")]
pub use cap_std_ext::cmdext::CapStdExtCommandExt;

#[cfg(not(target_os = "linux"))]
use std::os::fd::OwnedFd;
#[cfg(not(target_os = "linux"))]
use std::{process::Command, sync::Arc};

#[cfg(not(target_os = "linux"))]
pub trait CapStdExtCommandExt {
    #[allow(dead_code)]
    fn take_fd_n(&mut self, _fd: Arc<OwnedFd>, _num: i32) -> &mut Command;
}

#[cfg(not(target_os = "linux"))]
impl CapStdExtCommandExt for Command {
    fn take_fd_n(&mut self, _fd: Arc<OwnedFd>, _num: i32) -> &mut Command {
        self
    }
}
