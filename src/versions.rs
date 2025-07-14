//! Module containing code that handles versions.
use lenient_semver::Version;
use std::{
    convert::TryFrom,
    io::{Error, ErrorKind},
};

/// Type used for specifying the version of the installation.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct VsInstallationVersion<'a>(Version<'a>);

impl<'a> VsInstallationVersion<'a> {
    /// Parses the VsInstallationVersion from a string.
    pub fn parse(value: &'a str) -> std::io::Result<VsInstallationVersion<'a>> {
        Version::parse(value).map_or_else(
            |e| {
                Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Failed to parse &str as a VsInstallationVersion: {}", e),
                ))
            },
            |v| Ok(VsInstallationVersion(v)),
        )
    }

    /// Crate function for checking if the version is in the specified range.
    pub(crate) fn is_in_range(
        &self,
        max: Option<&VsInstallationVersion>,
        min: Option<&VsInstallationVersion>,
    ) -> bool {
        has_version_in_range(&self.0, max.map(|v| &v.0), min.map(|v| &v.0))
    }
}

/// Enum holding the VS product line versions.
pub enum VsProductLineVersion {
    Vs2022,
    Vs2019,
    Vs2017,
}

impl VsProductLineVersion {
    /// The non inclusive max installation version for a
    /// specific product line version.
    pub fn installation_version_max(&self) -> VsInstallationVersion {
        // Constant values that are always safe to parse.
        match self {
            Self::Vs2022 => VsInstallationVersion::parse("18.0.0.0").unwrap(),
            Self::Vs2019 => VsInstallationVersion::parse("17.0.0.0").unwrap(),
            Self::Vs2017 => VsInstallationVersion::parse("16.0.0.0").unwrap(),
        }
    }

    /// The inclusive min installation version for a
    /// specific product line version.
    pub fn installation_version_min(&self) -> VsInstallationVersion {
        match self {
            Self::Vs2022 => VsInstallationVersion::parse("17.0.0.0").unwrap(),
            Self::Vs2019 => VsInstallationVersion::parse("16.0.0.0").unwrap(),
            Self::Vs2017 => VsInstallationVersion::parse("15.0.0.0").unwrap(),
        }
    }
}

impl TryFrom<&str> for VsProductLineVersion {
    type Error = Error;

    fn try_from(s: &str) -> std::io::Result<Self> {
        match s {
            "2017" => Ok(VsProductLineVersion::Vs2017),
            "2019" => Ok(VsProductLineVersion::Vs2019),
            "2022" => Ok(VsProductLineVersion::Vs2022),
            _ => Err(Error::new(
                ErrorKind::InvalidData,
                format!("Product line version {} did not match any known values.", s),
            )),
        }
    }
}

/// The windows SDK version.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct WinSdkVersion<'a>(Version<'a>);

impl<'a> WinSdkVersion<'a> {
    pub fn parse(value: &'a str) -> std::io::Result<WinSdkVersion<'a>> {
        Version::parse(value).map_or_else(
            |e| {
                Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Failed to parse &str as a WinSdkVersion: {}", e),
                ))
            },
            |v| Ok(WinSdkVersion(v)),
        )
    }

    /// Crate function for checking if the version is in the specified range.
    pub(crate) fn is_in_range(
        &self,
        max: Option<&WinSdkVersion>,
        min: Option<&WinSdkVersion>,
    ) -> bool {
        has_version_in_range(&self.0, max.map(|v| &v.0), min.map(|v| &v.0))
    }
}

/// Internal function to check if a version is in the range
/// if it has been specified.
fn has_version_in_range(version: &Version, max: Option<&Version>, min: Option<&Version>) -> bool {
    let is_below_max: bool = max.map_or(true, |max_version| max_version > version);
    let is_above_min: bool = min.map_or(true, |min_version| version >= min_version);
    is_below_max && is_above_min
}

// ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Unit tests of the private functions and methods
// ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_msbuild_has_version_in_range() {
        let max = Some(
            Version::parse("4.3.2.1")
                .expect("It should be possible to create a Version object from the string 4.3.2.1"),
        );
        let min = Some(
            Version::parse("1.2.3.4")
                .expect("It should be possible to create a Version object from the string 1.2.3.4"),
        );
        // Check with no min or max
        assert!(
            has_version_in_range(
                &Version::parse("0.0.0.0").expect(
                    "It should be possible to create a Version object from the string 0.0.0.0"
                ),
                None,
                None
            ),
            "The version 0.0.0.0 should be in range when no min or max values have been specified."
        );
        // Check outside of range with min value.
        assert!(
            !has_version_in_range(
                &Version::parse("0.0.0.0").expect(
                    "It should be possible to create a Version object from the string 0.0.0.0"
                ),
                None,
                min.as_ref()
            ),
            "The version 0.0.0.0 should not be in range when min is 1.2.3.4"
        );
        // Check inside of range with min value
        assert!(
            has_version_in_range(
                &Version::parse("1.2.3.300").expect(
                    "It should be possible to create a Version object from the string 1.2.3.300"
                ),
                None,
                min.as_ref()
            ),
            "The version 1.2.3.300 should be in range when min is 1.2.3.4 and no max is given."
        );
        // Check out of range with max value
        assert!(
            !has_version_in_range(
                &Version::parse("4.3.2.11").expect(
                    "It should be possible to create a Version object from the string 4.3.2.11"
                ),
                max.as_ref(),
                None,
            ),
            "The version 4.3.2.11 should not be in range when max is 4.3.2.1 and no min is given."
        );
        // Check in range with max value
        assert!(
            has_version_in_range(
                &Version::parse("4.0.2.11").expect(
                    "It should be possible to create a Version object from the string 4.0.2.11"
                ),
                max.as_ref(),
                None,
            ),
            "The version 4.3.2.11 should not be in range when max is 4.3.2.1 and no min is given."
        );
        // Check in range with min and max
        assert!(
            has_version_in_range(
                &Version::parse("4.0.2.11").expect(
                    "It should be possible to create a Version object from the string 4.0.2.11"
                ),
                max.as_ref(),
                min.as_ref(),
            ),
            "The version 4.3.2.11 should not be in range when max is 4.3.2.1 and no max is given."
        );
    }
}
