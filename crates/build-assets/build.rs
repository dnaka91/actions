use std::path::PathBuf;

fn main() {
    let file = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("target.txt");
    let target = std::env::var("TARGET").unwrap();

    std::fs::write(file, target).unwrap();
}
