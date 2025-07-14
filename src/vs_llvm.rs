//! Module for llvm parts of a VS installation.
use crate::{vs_paths::sub_directory, VsInstallation};
use std::{
    convert::TryFrom,
    io::Error,
    path::{Path, PathBuf},
};

/// Type holding the paths associated with LLVM in the
/// Visual compiler tools.
pub struct VsLlvm {
    bin: PathBuf,
    lib: PathBuf,
    bin_x64: PathBuf,
    lib_x64: PathBuf,
}

impl VsLlvm {
    const BIN: &'static str = "VC/Tools/Llvm/bin";
    const LIB: &'static str = "VC/Tools/Llvm/lib";
    const BIN_X64: &'static str = "VC/Tools/Llvm/x64/bin";
    const LIB_X64: &'static str = "VC/Tools/Llvm/x64/lib";

    pub fn bin(&self) -> &Path {
        self.bin.as_ref()
    }

    pub fn lib(&self) -> &Path {
        self.lib.as_ref()
    }

    pub fn bin_x64(&self) -> &Path {
        self.bin_x64.as_ref()
    }

    pub fn lib_x64(&self) -> &Path {
        self.lib_x64.as_ref()
    }
}

impl TryFrom<&VsInstallation> for VsLlvm {
    type Error = Error;

    fn try_from(vs_installation: &VsInstallation) -> std::io::Result<VsLlvm> {
        Ok(VsLlvm {
            bin: sub_directory(vs_installation.path(), Self::BIN)?,
            lib: sub_directory(vs_installation.path(), Self::LIB)?,
            bin_x64: sub_directory(vs_installation.path(), Self::BIN_X64)?,
            lib_x64: sub_directory(vs_installation.path(), Self::LIB_X64)?,
        })
    }
}
