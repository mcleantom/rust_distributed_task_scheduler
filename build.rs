use std::{env, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    tonic_build::configure()
        .file_descriptor_set_path(out_dir.join("coordinator_descriptor.bin"))
        .compile(&["proto/coordinator.proto"], &["proto"])?;

    tonic_build::compile_protos("proto/coordinator.proto")?;

    Ok(())
}
