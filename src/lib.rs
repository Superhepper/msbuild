use std::{io::Error, path::PathBuf};

use serde_json::Value;

pub struct MsBuild {
    path: PathBuf,
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
            if let Some(ver) = ver {
                if version == ver {
                    return Ok(Self { path: pb} );
                }
            }
            else {
                return Ok(Self { path: pb} );
            }
        }
        Err(std::io::Result::Err(Error::other("Not found")))
    }

    pub fn run(&mut self, project_path: PathBuf, args: &[&str]) {
        let pb = self.path.join("MsBuild").join("Current").join("Bin");
        println!("Msbuild is in {:?}", pb);
        let output = std::process::Command::new(pb.join("MSBuild.exe"))
            .current_dir(project_path)
            .args(args)
            .output()
            .expect("Failed to run msbuild");
        let o = std::str::from_utf8(&output.stdout).unwrap();
        println!("{}", o);
    }
}
