fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Пересобираем, если изменился proto-файл
    println!("cargo:rerun-if-changed=proto/blog.proto");

    tonic_prost_build::configure()
        .build_client(true)
        .compile_protos(&["proto/blog.proto"], &["proto"])?;
    Ok(())
}
