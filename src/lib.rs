pub struct MsBuild {}

impl MsBuild {
    pub fn find_msbuild() -> Result<Self, std::io::Result<()>> {
        let output = std::process::Command::new(
            "C:\\Program Files (x86)\\Microsoft Visual Studio\\Installer\\vswhere.exe",
        )
        .args(["-legacy", "-prerelease", "-format", "json"])
        .output()
        .expect("Failed to run vswhere");
        println!("Output: {}", std::str::from_utf8(&output.stdout).unwrap());
        Ok(Self {})
    }
}
