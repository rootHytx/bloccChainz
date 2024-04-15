fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(false) // We don't need server code
        .compile(&["proto/kademlia.proto"], &["proto"])?;
    Ok(())
}