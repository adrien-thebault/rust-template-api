use glob::glob;
use std::{env, path::PathBuf};

fn main() -> std::io::Result<()> {
    tonic_prost_build::configure()
        .file_descriptor_set_path(
            PathBuf::from(env::var("OUT_DIR").unwrap()).join("descriptor.bin"),
        )
        // lets the `web` gateway crate (de)serialize these types directly as
        // JSON at the REST boundary instead of hand-writing a DTO per message.
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .compile_protos(
            &glob("src/**/*.proto")
                .expect("could not list proto files")
                .filter_map(Result::ok)
                .map(|p| p.to_string_lossy().to_string())
                .collect::<Vec<_>>(),
            &["src".to_string()],
        )?;

    Ok(())
}
