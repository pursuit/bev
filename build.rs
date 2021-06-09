fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(false)
        .out_dir("src/pursuit/api/mortalkin")
        .compile(
            &["mortalkin/user.proto", "mortalkin/game.proto"],
            &["../shared/proto/api"],
        )?;

    Ok(())
}
