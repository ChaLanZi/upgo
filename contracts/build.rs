fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .out_dir("src/proto")
        .compile_protos(
            &[
                "proto/user.proto",
                "proto/fund.proto",
                "proto/position.proto",
                "proto/risk.proto",
                "proto/auth.proto",
            ],
            &["proto"],
        )?;
    Ok(())
}
