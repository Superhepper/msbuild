//! Module for code related to a full installation of VS or just
//! the VS build tools.
use crate::{versions::VsInstallationVersion, vs_where::VsWhere};
use serde_json::Value;
use std::{
    io::{Error, ErrorKind},
    path::{Path, PathBuf},
};

/// Type containing information about the installation.
pub struct VsInstallation {
    path: PathBuf,
}

impl VsInstallation {
    const ENV_KEY: &'static str = "VS_INSTALLATION_PATH";

    /// The path of the VS installation.
    pub fn path(&self) -> &Path {
        self.path.as_path()
    }

    /// Finds a VS installation with the highest installation version that is in a range
    /// between max (exclusive) and min(inclusive).
    /// # Examples
    ///
    /// ```
    /// // Find the latest supported version for msbuild
    /// use msbuild::installation::{VsInstallationVersion, VsInstallation};
    ///
    /// let max = Some(VsInstallationVersion::parse("17.10.35013.160").unwrap());
    /// let min = Some(VsInstallationVersion::parse("17.0.0.0").unwrap());
    ///
    /// let vs_installation = VsInstallation::find_in_range(max, min);
    /// ```
    pub fn find_in_range(
        max: Option<VsInstallationVersion>,
        min: Option<VsInstallationVersion>,
    ) -> std::io::Result<Self> {
        VsWhere::find_vswhere()
            .and_then(|vswhere| vswhere.run(None))
            .and_then(|output| Self::parse_from_json(&output))
            .and_then(|v: Value| {
                Self::list_instances(&v)
                    .and_then(|instances| Self::find_match(instances, max.as_ref(), min.as_ref()))
            })
            .map(|path| VsInstallation { path })
    }

    // Internal function for finding the instances that matches the
    // version range and, if specified, the path in the environment
    // variable.
    fn find_match(
        instances_json: &[Value],
        max: Option<&VsInstallationVersion>,
        min: Option<&VsInstallationVersion>,
    ) -> std::io::Result<PathBuf> {
        let env_installation_path: Option<PathBuf> =
            std::env::var(Self::ENV_KEY).ok().map(|v| PathBuf::from(&v));

        // Parse the instance json data and filter result based on version.
        let validated_instances = Self::validate_instances_json(instances_json, max, min);

        if let Some(specified_installation_path) = env_installation_path {
            // Finds the specified installation path among the parsed
            // and validated instances.
            validated_instances
                .iter()
                .filter_map(|(_, p)| {
                    if specified_installation_path.starts_with(p) {
                        Some(p.to_path_buf())
                    } else {
                        None
                    }
                })
                .next()
                .ok_or(Error::new(
                    ErrorKind::NotFound,
                    "No instance found that matched requirements.",
                ))
        } else {
            // Select the latest version.
            validated_instances
                .iter()
                .max_by_key(|(v, _)| v)
                .map(|(_, p)| p.to_path_buf())
                .ok_or(Error::new(
                    ErrorKind::NotFound,
                    "No instance found that matched requirements.",
                ))
        }
    }

    /// Internal function that extracts a collection of parsed
    /// installation instances with a version within the given
    /// interval.
    fn validate_instances_json<'a>(
        instances_json: &'a [Value],
        max: Option<&'a VsInstallationVersion>,
        min: Option<&'a VsInstallationVersion>,
    ) -> Vec<(VsInstallationVersion<'a>, &'a Path)> {
        instances_json
            .iter()
            .filter_map(|i| {
                VsInstallation::parse_installation_version(i)
                    .and_then(|installation_version| {
                        if installation_version.is_in_range(max, min) {
                            VsInstallation::parse_installation_path(i).map(|installation_path| {
                                Some((installation_version, installation_path))
                            })
                        } else {
                            // Maybe log(trace) that an instance was found that was not in the range.
                            Ok(None)
                        }
                    })
                    .unwrap_or_else(|e| {
                        print!("Encounted an error during parsing of instance data: {}", e);
                        None
                    })
            })
            .collect()
    }

    // Internal function for parsing a string as json object.
    fn parse_from_json(value: &str) -> std::io::Result<Value> {
        serde_json::from_str(value).map_err(|e| {
            Error::new(
                ErrorKind::InvalidData,
                format!("Failed to parse command output as json ({})", e),
            )
        })
    }

    // Internal function for listing the instances inthe json value.
    fn list_instances(v: &Value) -> std::io::Result<&Vec<Value>> {
        v.as_array().ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidData,
                "json data did not contain any installation instances.",
            )
        })
    }

    /// Function for parsing the installation path from
    /// the return value of `vs_where`.
    fn parse_installation_path(json_value: &Value) -> std::io::Result<&Path> {
        json_value
            .get("installationPath")
            .and_then(|path_json_value: &Value| path_json_value.as_str())
            .ok_or(Error::new(
                ErrorKind::InvalidData,
                "Failed to retrieve `installationPath`.",
            ))
            .map(Path::new)
    }

    /// Function for parsing the installation version from
    /// the return value of `vs_where`.
    fn parse_installation_version(
        json_value: &Value,
    ) -> std::io::Result<VsInstallationVersion<'_>> {
        json_value
            .get("installationVersion")
            .and_then(|version_json_value: &Value| version_json_value.as_str())
            .and_then(|version_str: &str| VsInstallationVersion::parse(version_str).ok())
            .ok_or(Error::new(
                ErrorKind::InvalidData,
                "Failed to retrieve `installationVersion`.",
            ))
    }
}

// ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Unit tests of the private functions and methods
// ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_installation_version() {
        let version_str = "2.3.1.34";
        let json_value = serde_json::json!({
            "instanceId": "VisualStudio.14.0",
            "installationPath": "C:\\Program Files (x86)\\Microsoft Visual Studio 14.0\\",
            "installationVersion": version_str
        });
        let expected = VsInstallationVersion::parse(version_str)
            .expect("It should be possible to parse the `version_str` as Version object.");
        let actual = VsInstallation::parse_installation_version(&json_value).expect(
            "The function should be to extract an installation version from the json_value.",
        );
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parse_installation_path() {
        let expected = Path::new("C:\\Program Files (x86)\\Microsoft Visual Studio 14.0\\");
        let json_value = serde_json::json!({
            "instanceId": "019109ba",
            "installDate": "2023-08-26T14:05:02Z",
            "installationName": "VisualStudio/17.12.0+35506.116",
            "installationPath": expected.to_string_lossy(),
            "installationVersion": "17.12.35506.116",
            "productId": "Microsoft.VisualStudio.Product.Community",
            "productPath": "C:\\Program Files\\Microsoft Visual Studio\\2022\\Community\\Common7\\IDE\\devenv.exe",
        });
        let actual = VsInstallation::parse_installation_path(&json_value)
            .expect("The function should be to extract an installation path from the json_value.");
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_msbuild_validate_instances_json() {
        let json_value = serde_json::json!([
            {
                "installationPath": "C:\\Program Files (x86)\\Microsoft Visual Studio 14.0\\",
                "installationVersion": "14.0",
            },
            {
                "installationPath": "C:\\Program Files\\Microsoft Visual Studio\\2022\\Community",
                "installationVersion": "17.12.35506.116",
            },
            {
                "installationPath": "C:\\Program Files\\Microsoft Visual Studio\\2022\\Enterprise",
                "installationVersion": "17.08.35506.116",
            },
        ]);

        let values: &Vec<Value> = json_value
            .as_array()
            .expect("It should be possible to parse the json as an array of objects.");

        // Sanity check.
        assert_eq!(
            values.len(),
            3,
            "There should be 3 instances: \n {:?}",
            values
        );

        let min = Some(
            VsInstallationVersion::parse("17.9")
                .expect("It should be possible to parse the 17.9 as a version."),
        );
        let max = Some(
            VsInstallationVersion::parse("18.0")
                .expect("It should be possible to parse the 18.0 as a version."),
        );
        let validated_instances =
            VsInstallation::validate_instances_json(values.as_slice(), max.as_ref(), min.as_ref());
        let expected_version = VsInstallationVersion::parse("17.12.35506.116")
            .expect("It should be possible to parse avlid version.");
        let expected_path =
            Path::new("C:\\Program Files\\Microsoft Visual Studio\\2022\\Community");
        assert_eq!(
            validated_instances.len(),
            1,
            "There should only be 1 element found."
        );
        let (actual_version, actual_path) = validated_instances.first().unwrap();
        assert_eq!(
            expected_version, *actual_version,
            "The returned version was not the expected one",
        );
        assert_eq!(
            expected_path, *actual_path,
            "The returned path was not the expected one."
        );
    }

    #[test]
    fn test_msbuild_find_match() {
        let json_value = serde_json::json!([
            {
                "installationPath": "C:\\Program Files (x86)\\Microsoft Visual Studio 14.0\\",
                "installationVersion": "14.0",
            },
            {
                "installationPath": "C:\\Program Files\\Microsoft Visual Studio\\2022\\Community",
                "installationVersion": "17.12.35506.116",
            },
            {
                "installationPath": "C:\\Program Files\\Microsoft Visual Studio\\2022\\Enterprise",
                "installationVersion": "17.08.35506.116",
            },
        ]);

        let values: &Vec<Value> = json_value
            .as_array()
            .expect("It should be possible to parse the json as an array of objects.");

        // Sanity check.
        assert_eq!(
            values.len(),
            3,
            "There should be 3 instances: \n {:?}",
            values
        );

        // The min and max are now chosen so that they will include
        // two possible result.
        let min = Some(
            VsInstallationVersion::parse("17.7")
                .expect("It should be possible to parse the 17.9 as a version."),
        );
        let max = Some(
            VsInstallationVersion::parse("18.0")
                .expect("It should be possible to parse the 18.0 as a version."),
        );

        // The expected values, when no environment variable have been set,
        // is the one with the latest version.
        let expected = PathBuf::from("C:\\Program Files\\Microsoft Visual Studio\\2022\\Community");

        let actual = VsInstallation::find_match(values, max.as_ref(), min.as_ref())
            .expect("The function is expected to return a valid result.");

        assert_eq!(
            expected, actual,
            "The resulting path does not match the expected one."
        );
    }
}
