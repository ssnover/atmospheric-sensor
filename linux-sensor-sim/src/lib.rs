use std::fs::read_link;
use std::io::Result;
use std::os::unix::{fs::symlink, io::FromRawFd};
use std::path::{Path, PathBuf};

use nix::pty;
use tokio::fs::File;

pub struct PseudoTerminal {
    pub master_file: File,
    pub master_path: PathBuf,
    pub slave_file: File,
    pub slave_path: PathBuf,
}

impl PseudoTerminal {
    pub fn new() -> Result<Self> {
        let pty_pair = pty::openpty(None, None).unwrap();
        Ok(PseudoTerminal {
            master_file: unsafe { File::from_raw_fd(pty_pair.master) },
            master_path: read_link(format!("/proc/self/fd/{}", pty_pair.master))?,
            slave_file: unsafe { File::from_raw_fd(pty_pair.slave) },
            slave_path: read_link(format!("/proc/self/fd/{}", pty_pair.slave))?,
        })
    }

    pub fn create_symlink(&self, symlink_path: &Path) -> Result<()> {
        if symlink_path.exists() {
            std::fs::remove_file(symlink_path)?;
        }
        symlink(&self.slave_path, symlink_path)?;
        Ok(())
    }
}
