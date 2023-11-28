use std::{
    collections::HashSet,
    io::Error,
    os::fd::{AsRawFd, RawFd},
};

use libc::nfds_t;

pub struct ReadPoller {
    fds: Vec<libc::pollfd>,
    readable_fds: HashSet<RawFd>,
}

impl ReadPoller {
    pub fn new() -> Self {
        Self {
            fds: Vec::new(),
            readable_fds: HashSet::new(),
        }
    }

    pub fn add_read(&mut self, fd: &impl AsRawFd) {
        self.fds.push(libc::pollfd {
            fd: fd.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        });
    }

    pub fn poll(&mut self) -> Result<(), String> {
        let result = unsafe { libc::poll(self.fds.as_mut_ptr(), self.fds.len() as nfds_t, -1) };
        if result < 0 {
            return Err(format!("poll() failed: {}", Error::last_os_error()));
        }

        for fd in &self.fds {
            if fd.revents & libc::POLLIN != 0 {
                self.readable_fds.insert(fd.fd);
            }
        }

        Ok(())
    }

    pub fn is_readable(&self, fd: &impl AsRawFd) -> bool {
        self.readable_fds.contains(&fd.as_raw_fd())
    }

    pub fn clear(&mut self) {
        self.fds.clear();
        self.readable_fds.clear();
    }

    pub fn reset(&mut self) {
        for fd in &mut self.fds {
            fd.revents = 0;
        }
        self.readable_fds.clear();
    }
}
