use msbuild::{InstallationVersion, MsBuild, ProductLineVersion};

#[ignore]
#[test]
fn test_find_msbuild() {
    // This will only work if the specified version
    // is installed in the CI environment and is recognised
    // by vswhere.

    // Find specific version 2022.
    assert!(MsBuild::find_msbuild(Some("2022")).is_ok());

    // Find any version available on the system
    assert!(MsBuild::find_msbuild(None).is_ok());
}

#[ignore]
#[test]
fn test_find_msbuild_in_range_with_installed_version_in_range() {
    // This will only work if any edition of Visual Studio 2022
    // is installed in the CI environment.

    assert!(MsBuild::find_msbuild_in_range(
        Some(ProductLineVersion::Vs2022.installation_version_max()),
        Some(ProductLineVersion::Vs2022.installation_version_min())
    )
    .is_ok());
}

#[ignore]
#[test]
fn test_find_msbuild_with_installed_version_out_of_range() {
    let invalid_min: InstallationVersion = InstallationVersion::parse("1000.0.0.0")
        .expect("Should be possible to parse valid version string");

    let invalid_min_result = MsBuild::find_msbuild_in_range(None, Some(invalid_min.clone()));
    assert!(invalid_min_result.is_err(), "Providing the function with a min version that would prevent it to find any products should result in an error.");

    let invalid_max = InstallationVersion::parse("0.0.0.1")
        .expect("Should be possible to parse valid version string");
    let invalid_max_result = MsBuild::find_msbuild_in_range(Some(invalid_max.clone()), None);
    assert!(invalid_max_result.is_err(), "Providing the function with a max version that would prevent it to find any products should result in an error.");

    let invalid_range_result = MsBuild::find_msbuild_in_range(Some(invalid_max), Some(invalid_min));
    assert!(invalid_range_result.is_err(), "Providing the function with a min and a max version that would prevent it to find any products should result in an error.");
}
