#![cfg_attr(feature = "never", feature(never_type))]

use std::io::{
    Write
};
use std::fs::{self, File};
use std::env;
use std::ffi::{CString, NulError};
use log::info;
use nix::unistd::{execve, unlink};

#[cfg(feature = "never")]
type ReturnType = !;
#[cfg(not(feature = "never"))]
type ReturnType = ();

pub fn update_self_exe(new_binary: &[u8]) -> anyhow::Result<ReturnType> {
    let self_path = env::current_exe()?.canonicalize()?;
    let perms = fs::metadata(&self_path)?.permissions();
    log::info!("Unlinking original executable @ '{}'", self_path.to_string_lossy());
    unlink(&self_path)?;
    log::info!("Creating new file");
    let mut file = File::create(&self_path)?;
    fs::set_permissions(&self_path, perms)?;
    log::info!("Writing file data");
    file.write_all(new_binary)?;
    drop(file);

    let args: Vec<CString> = env::args().map(CString::new).collect::<Result<Vec<CString>, NulError>>()?;
    let env: Vec<CString> = env::vars().map(|(v1, v2)| format!("{}={}", v1, v2)).map(CString::new).collect::<Result<Vec<CString>, NulError>>()?;
    let path: CString = CString::new(self_path.to_str().ok_or(anyhow::Error::msg("Failed to convert path to string!"))?.to_string())?;

    info!("Replacing current process with the new executable");
    execve(&path, &args, &env)?;
    unreachable!()
}
