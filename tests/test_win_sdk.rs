use msbuild::win_sdk::{WinSdk, WinSdkVersion};
use std::path::Path;
use tempfile::tempdir;

struct CategoryFolder {
    name: &'static str,
    versioned_subdirs: &'static [&'static str],
    expected_dirs: &'static [&'static str],
}

// This is used to create a valid layout inside the installation
// folder.
const CATEGORY_FOLDERS: [CategoryFolder; 8] = [
    CategoryFolder {
        name: "bin",
        versioned_subdirs: &[
            "10.0.14393.0",
            "10.0.15063.0",
            "10.0.16299.0",
            "10.0.17134.0",
            "10.0.18362.0",
            "10.0.19041.0",
            "10.0.20348.0",
            "10.0.22000.0",
            "10.0.22621.0",
        ],
        expected_dirs: &["x86", "x64"],
    },
    CategoryFolder {
        name: "Include",
        versioned_subdirs: &[
            "10.0.10240.0",
            "10.0.18362.0",
            "10.0.19041.0",
            "10.0.20348.0",
            "10.0.22000.0",
            "10.0.22621.0",
        ],
        expected_dirs: &["cppwinrt", "shared", "ucrt", "um", "winrt"],
    },
    CategoryFolder {
        name: "Lib",
        versioned_subdirs: &[
            "10.0.10240.0",
            "10.0.18362.0",
            "10.0.19041.0",
            "10.0.20348.0",
            "10.0.22000.0",
            "10.0.22621.0",
        ],
        expected_dirs: &["ucrt", "ucrt_enclave", "um"],
    },
    CategoryFolder {
        name: "Licenses",
        versioned_subdirs: &[
            "10.0.18362.0",
            "10.0.19041.0",
            "10.0.20348.0",
            "10.0.22000.0",
            "10.0.22621.0",
        ],
        expected_dirs: &[],
    },
    CategoryFolder {
        name: "Redist",
        versioned_subdirs: &[
            "10.0.18362.0",
            "10.0.19041.0",
            "10.0.20348.0",
            "10.0.22000.0",
            "10.0.22621.0",
        ],
        expected_dirs: &["ucrt"],
    },
    CategoryFolder {
        name: "References",
        versioned_subdirs: &[
            "10.0.18362.0",
            "10.0.19041.0",
            "10.0.20348.0",
            "10.0.22000.0",
            "10.0.22621.0",
        ],
        expected_dirs: &[],
    },
    CategoryFolder {
        name: "Source",
        versioned_subdirs: &[
            "10.0.10240.0",
            "10.0.18362.0",
            "10.0.19041.0",
            "10.0.20348.0",
            "10.0.22000.0",
            "10.0.22621.0",
        ],
        expected_dirs: &["ucrt"],
    },
    CategoryFolder {
        name: "UnionMetadata",
        versioned_subdirs: &[
            "10.0.18362.0",
            "10.0.19041.0",
            "10.0.20348.0",
            "10.0.22000.0",
            "10.0.22621.0",
        ],
        expected_dirs: &["facade"],
    },
];

fn setup_installation_folder(dst: &Path) {
    CATEGORY_FOLDERS.iter().for_each(|cf| {
        let category_folder_path = dst.join(cf.name);
        // Create category folders
        std::fs::create_dir(category_folder_path.as_path()).unwrap_or_else(|_| {
            panic!("It should be possible to create the `{}` dir.", cf.name);
        });
        cf.versioned_subdirs.iter().for_each(|versioned_subdir| {
            // Create versioned subdirs
            let versioned_sub_dir = category_folder_path.as_path().join(versioned_subdir);
            std::fs::create_dir(versioned_sub_dir.as_path()).unwrap_or_else(|_| {
                panic!("It should be possible to create the versioned subdir `{}` inside the `{}` dir.", versioned_subdir, cf.name);
            });
            cf.expected_dirs.iter().for_each(|expected_dir| {
                // Create expected dirs.
                std::fs::create_dir(versioned_sub_dir.as_path().join(expected_dir)).unwrap_or_else(|_| {
                    panic!("It should be possible to create the expected dir `{}` inside the versioned subdir `{}`, inside the `{}` dir.", expected_dir, versioned_subdir, cf.name);
                });
            });
        });

    });
}

#[test]
fn test_find() {
    let temp_dir = tempdir().expect("It should be possible to create a temporary directory.");
    let installation_path = temp_dir.path();
    setup_installation_folder(installation_path);
    // This function is only safe to call on windows
    // in a single threaded context.
    unsafe {
        std::env::set_var("WIN_SDK_PATH", installation_path.as_os_str());
    }
    let actual = WinSdk::find().expect(
        "It should be possible to find a WinSdk in the properly setup installation directory.",
    );

    let expected_versioned_dir = installation_path.join("Include/10.0.22621.0");
    assert_eq!(
        expected_versioned_dir.join("cppwinrt").as_path(),
        actual.include_dirs().cppwinrt_dir()
    );
}

#[test]
fn test_find_in_range() {
    let temp_dir = tempdir().expect("It should be possible to create a temporary directory.");
    let installation_path = temp_dir.path();
    setup_installation_folder(installation_path);
    // This function is only safe to call on windows
    // in a single threaded context.
    unsafe {
        std::env::set_var("WIN_SDK_PATH", installation_path.as_os_str());
    }

    let min_version = WinSdkVersion::parse("10.0.20000.0")
        .expect("It should be possible to parse a valid version");
    let max_version = WinSdkVersion::parse("10.0.21000.0")
        .expect("It should be possible to parse a valid version");

    let actual = WinSdk::find_in_range(Some(max_version), Some(min_version)).expect(
        "It should be possible to find a WinSdk in range in the properly setup installation directory.",
    );

    let expected_versioned_dir = installation_path.join("Include/10.0.20348.0");
    assert_eq!(
        expected_versioned_dir.join("cppwinrt").as_path(),
        actual.include_dirs().cppwinrt_dir()
    );
}
