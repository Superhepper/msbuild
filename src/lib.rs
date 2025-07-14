//! # The msbuild crate
//! This crates provides the functionality of finding
//! the msbuild binary on the system.
//!
//! But it also provides functionality for finding other
//! paths that may be needed when using msbuild e.g.
//! WinSDK.
//!
//! # Environment Variables
//! - The `VS_WHERE_PATH` environment variable can be used in order
//!   overwrite the default path where the crate tries to locate
//!   the `vswhere.exe` binary.
//!
//! - The `VS_INSTALLATION_PATH` environment variable can be used in order
//!   to overwrite specify a path to Visual Studio installation
//!   Note! The path must still lead to an installation that fulfills the version
//!   requirements otherwise the crate will try to probe the system
//!   for a suitable version.
//!
//! - The `WIN_SDK_PATH` environment variable can be used in order to
//!   to overwrite in what location the library will search for
//!   WinSDK installations.
use std::{
    convert::TryFrom,
    io::{Error, ErrorKind},
    path::{Path, PathBuf},
};

mod versions;

pub(crate) mod vs_paths;

pub mod vs_installation;
pub mod vs_llvm;
pub mod vs_where;
pub mod win_sdk;

pub use versions::{VsInstallationVersion, VsProductLineVersion};
pub use vs_installation::VsInstallation;
pub use vs_llvm::VsLlvm;
pub use vs_where::VsWhere;

/// Type for finding and interactive with
/// the msbuild executable.
pub struct MsBuild {
    path: PathBuf,
}

impl MsBuild {
    /// Finds the msbuild executable that is associated with provided product line version
    /// if no version is provided then the first installation of msbuild that is found
    /// will be selected.
    ///
    /// # Examples
    ///
    /// ```
    /// use msbuild::MsBuild;
    ///
    /// let product_line_version: Option<&str> = Some("2017");
    /// let msbuild: MsBuild = MsBuild::find_msbuild(product_line_version)
    ///     .expect("A 2017 VS installation should exist");
    /// ```
    pub fn find_msbuild(product_line_version: Option<&str>) -> std::io::Result<Self> {
        product_line_version
            .map(VsProductLineVersion::try_from)
            .transpose()
            .and_then(|potential_plv| {
                let max = potential_plv
                    .as_ref()
                    .map(|plv| plv.installation_version_max());
                let min = potential_plv
                    .as_ref()
                    .map(|plv| plv.installation_version_min());
                MsBuild::find_msbuild_in_range(max, min)
            })
    }

    /// Finds a msbuild with the highest installation version that is in a range
    /// between max (exclusive) and min(inclusive).
    ///
    /// # Examples
    ///
    /// ```
    /// // Find the latest supported version for msbuild
    /// use msbuild::{MsBuild, VsProductLineVersion};
    ///
    /// let msbuild = MsBuild::find_msbuild_in_range(
    ///     Some(VsProductLineVersion::Vs2022.installation_version_max()),
    ///     Some(VsProductLineVersion::Vs2017.installation_version_min()),
    /// );
    /// ```
    pub fn find_msbuild_in_range(
        max: Option<VsInstallationVersion>,
        min: Option<VsInstallationVersion>,
    ) -> std::io::Result<Self> {
        VsInstallation::find_in_range(max, min)
            .and_then(|vs_installation| Self::try_from(&vs_installation))
    }

    /// Executes msbuild using the provided project_path and
    /// the provided arguments.
    pub fn run(&self, project_path: &Path, args: &[&str]) -> std::io::Result<()> {
        if !self.path.as_path().exists() {
            return Err(Error::new(
                ErrorKind::NotFound,
                format!("Could not find [{}].", self.path.to_string_lossy()),
            ));
        }
        std::process::Command::new(self.path.as_path())
            .current_dir(project_path)
            .args(args)
            .output()
            .and_then(|out| {
                if out.status.success() {
                    Ok(())
                } else {
                    use std::io::Write;
                    std::io::stdout().write_all(&out.stdout)?;
                    let error_message = if let Some(code) = out.status.code() {
                        &format!("Failed to run msbuild: Exit code [{code}]")
                    } else {
                        "Failed to run msbuild"
                    };
                    Err(Error::new(
                        ErrorKind::Other,
                        format!("Failed to run msbuild: {error_message}"),
                    ))
                }
            })
    }
}

impl TryFrom<&VsInstallation> for MsBuild {
    type Error = Error;

    fn try_from(vs_installation: &VsInstallation) -> std::io::Result<MsBuild> {
        let path: PathBuf = vs_installation
            .path()
            .join("MsBuild/Current/Bin/msbuild.exe");
        if !path.is_file() {
            return Err(Error::new(
                ErrorKind::NotFound,
                format!("No msbuild executable found at {}", path.display()),
            ));
        }
        Ok(MsBuild { path })
    }
}

// ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Unit tests of the private functions and methods
// ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
mod test {}
