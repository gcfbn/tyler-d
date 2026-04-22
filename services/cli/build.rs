fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .compile(
            &["../../protos/tscherepacha.proto"],
            &["../../protos"],
        )?;
    Ok(())
}
