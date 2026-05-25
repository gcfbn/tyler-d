fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .compile(
            &[
                "../../protos/ocr.proto",
                "../../protos/tyler_d.proto",
                "../../protos/llm.proto"
            ],
            &["../../protos"],
        )?;
    Ok(())
}
