use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    // Get the directory where the build script is being run
    let out_dir = env::var("OUT_DIR").unwrap();

    // Define the source and destination paths
    let src_dir = PathBuf::from("assets");
    let dest_dir = PathBuf::from(out_dir).join("../../../assets");

    // Create the destination directory if it doesn't exist
    fs::create_dir_all(&dest_dir).unwrap();

    // Copy each file from the source directory to the destination directory
    for entry in fs::read_dir(src_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            let file_name = path.file_name().unwrap();
            fs::copy(&path, dest_dir.join(file_name)).unwrap();
        }
    }
}