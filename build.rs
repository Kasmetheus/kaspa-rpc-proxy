fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build protobuf definitions from local proto/
    tonic_build::configure()
        .build_server(false)
        .build_client(true)
        .compile(
            &[
                "proto/rpc.proto",
                "proto/messages.proto",
            ],
            &["proto"],
        )?;
    Ok(())
}
