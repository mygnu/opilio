fn main() {
    println!("cargo:rerun-if-changed=/assets/icons/opilio_64x64.png");

    let image = image::open(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/icons/opilio_64x64.png"
    ))
    .expect("Failed to open icon path")
    .into_rgba8()
    .into_raw();
    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let dest_path = std::path::Path::new(&out_dir).join("icon.bin");
    std::fs::write(dest_path, image).unwrap();
}
