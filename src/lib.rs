use std::{io::Error, num::ParseFloatError, path::PathBuf, process::ExitStatus};

use serde_json::Value;

pub struct MsBuild {
    path: PathBuf,
    install: PathBuf,
}

impl MsBuild {
    pub fn find_msbuild(ver: Option<&str>) -> Result<Self, std::io::Result<()>> {
        let output = std::process::Command::new(
            "C:\\Program Files (x86)\\Microsoft Visual Studio\\Installer\\vswhere.exe",
        )
        .args(["-legacy", "-prerelease", "-format", "json"])
        .output()
        .expect("Failed to run vswhere");
        let o = std::str::from_utf8(&output.stdout).unwrap();
        let v: Value = serde_json::from_str(o).unwrap();
        for c in v.as_array().unwrap().iter() {
            let catalog = c.get("catalog").unwrap();
            let version = catalog.get("productLineVersion").unwrap();
            let p = c.get("installationPath").unwrap();
            let pb: PathBuf = PathBuf::from(p.as_str().unwrap());
            let p2 = c.get("resolvedInstallationPath").unwrap();
            let pb2 = PathBuf::from(p2.as_str().unwrap());
            if let Some(ver) = ver {
                if version == ver {
                    println!("Options: {:?}", c);
                    return Ok(Self {
                        path: pb,
                        install: pb2,
                    });
                }
            } else {
                return Ok(Self {
                    path: pb,
                    install: pb2,
                });
            }
        }
        Err(std::io::Result::Err(Error::other("Not found")))
    }

    pub fn run(&mut self, project_path: PathBuf, args: &[&str]) {
        let mut pb = self.path.join("MsBuild");

        for els in std::fs::read_dir(&pb).unwrap() {
            let name = els.unwrap().file_name();
            let name = name.to_str().unwrap();
            let version: Result<f32, ParseFloatError> = name.parse();
            if version.is_ok() {
                pb = pb.join(name);
            }
            if name == "Current" {
                pb = pb.join(name);
            }
        }
        let pb = pb.join("Bin");
        let output = std::process::Command::new(pb.join("MSBuild.exe"))
            .current_dir(project_path)
            .args(args)
            .output()
            .expect("Failed to run msbuild");
        let o = std::str::from_utf8(&output.stdout).unwrap();
        println!("{}", o);
        if let Some(c) = output.status.code() {
            panic!("Failed to run build {c}");
        }
    }
}
