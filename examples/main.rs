use msbuild::MsBuild;

fn main() {
    let mb = MsBuild::find_msbuild();
    match mb {
        Ok(mut msb) => {
            msb.run("./".into(), &[]);
            println!("Found msbuild");
        }
        Err(_) => {
            println!("Failed to find msbuild");
        }
    }
}
