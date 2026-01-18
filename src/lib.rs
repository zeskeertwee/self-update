#![cfg_attr(feature = "never", feature(never_type))]

use log::info;
use nix::unistd::{execve, unlink};
use std::env;
use std::ffi::{CString, NulError};
use std::fs::{self, File};
use std::io::Write;

#[cfg(feature = "never")]
type ReturnType = !;
#[cfg(not(feature = "never"))]
type ReturnType = ();

pub fn update_self_exe(new_binary: &[u8]) -> Result<ReturnType, SelfUpdateError> {
    let self_path = env::current_exe()
        .map_err(|e| SelfUpdateError::Io(e))?
        .canonicalize()
        .map_err(|e| SelfUpdateError::Io(e))?;
    let perms = fs::metadata(&self_path)
        .map_err(|e| SelfUpdateError::Io(e))?
        .permissions();
    log::info!(
        "Unlinking original executable @ '{}'",
        self_path.to_string_lossy()
    );
    unlink(&self_path).map_err(|e| SelfUpdateError::FailUnlink(e))?;
    log::info!("Creating new file");
    let mut file = File::create(&self_path).map_err(|e| SelfUpdateError::Io(e))?;
    fs::set_permissions(&self_path, perms).map_err(|e| SelfUpdateError::Io(e))?;
    log::info!("Writing file data");
    file.write_all(new_binary)
        .map_err(|e| SelfUpdateError::Io(e))?;
    drop(file);

    let args: Vec<CString> = env::args()
        .map(CString::new)
        .collect::<Result<Vec<CString>, NulError>>()
        .map_err(|e| SelfUpdateError::NulError(e))?;
    let env: Vec<CString> = env::vars()
        .map(|(v1, v2)| format!("{}={}", v1, v2))
        .map(CString::new)
        .collect::<Result<Vec<CString>, NulError>>()
        .map_err(|e| SelfUpdateError::NulError(e))?;
    let path: CString = CString::new(
        self_path
            .to_str()
            .ok_or(SelfUpdateError::InvalidPath)?
            .to_string(),
    ).map_err(|e| SelfUpdateError::NulError(e))?;

    info!("Replacing current process with the new executable");
    execve(&path, &args, &env).map_err(|e| SelfUpdateError::FailExecute(e))?;
    unreachable!()
}

#[derive(Debug)]
pub enum SelfUpdateError {
    FailUnlink(nix::Error),
    FailExecute(nix::Error),
    NulError(NulError),
    Io(std::io::Error),
    InvalidPath,
}

impl std::error::Error for SelfUpdateError {}

impl std::fmt::Display for SelfUpdateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FailUnlink(e) => write!(f, "failed to unlink executable because: {}", e),
            Self::FailExecute(e) => write!(f, "failed to execute new executable because: {}", e),
            Self::NulError(e) => write!(f, "{}", e),
            Self::Io(e) => write!(f, "{}", e),
            Self::InvalidPath => write!(f, "Failed to convert path to string")
        }
    }
}
