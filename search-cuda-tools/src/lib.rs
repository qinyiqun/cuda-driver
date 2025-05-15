use std::{env::var_os, path::PathBuf};

/// Returns the path to the mx home directory, if it is set.
#[inline]
pub fn find_cuda_root() -> Option<PathBuf> {
    var_os("MACA_PATH").map(PathBuf::from)
}
