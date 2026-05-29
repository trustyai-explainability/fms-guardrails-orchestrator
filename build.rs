use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Skip proto compilation if generated code already exists.
    // The generated files in src/pb/ are committed to the repo,
    // so protoc is only needed when .proto files change.
    // To regenerate: delete src/pb/ and rebuild with protoc installed.
    if Path::new("src/pb/mod.rs").exists() {
        return Ok(());
    }
    std::fs::create_dir("src/pb").unwrap_or(());
    tonic_prost_build::configure()
        .build_client(true)
        .build_server(false)
        .out_dir("src/pb")
        .include_file("mod.rs")
        .compile_protos(
            &[
                "protos/caikit_runtime_Chunkers.proto",
                "protos/caikit_runtime_Nlp.proto",
                "protos/generation.proto",
                "protos/caikit_data_model_caikit_nlp.proto",
                "protos/health_check.proto",
            ],
            &["protos"],
        )
        .unwrap_or_else(|e| panic!("protobuf compilation failed: {e}"));

    Ok(())
}
