use msbuild::MsBuild;

fn main() {
    let mb = MsBuild::find_msbuild();
    match mb {
        Ok(msb) => {
            println!("Found msbuild");
        }
        Err(_) => {
            println!("Failed to find msbuild");
        }
    }
}
