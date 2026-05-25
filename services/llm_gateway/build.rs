fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .compile(
            &["../../protos/llm.proto"],
            &["../../protos"],
        )?;
    Ok(())
}
