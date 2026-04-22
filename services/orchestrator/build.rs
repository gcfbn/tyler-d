fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .compile(
            &["../../protos/ocr.proto"],
            &["../../protos"],
        )?;
    Ok(())
}
