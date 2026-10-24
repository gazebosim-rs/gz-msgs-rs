fn main() {
    let mut schemas: Vec<_> = std::fs::read_dir("schemas/gz/msgs")
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "proto")
        })
        .collect();
    schemas.sort();
    let registry = schemas
        .iter()
        .map(|path| {
            format!(
                "{}::file_descriptor().clone(),",
                path.file_stem().unwrap().to_str().unwrap()
            )
        })
        .collect::<String>();
    std::fs::write(
        std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("registry.rs"),
        format!("pub fn files() -> Vec<protobuf::reflect::FileDescriptor> {{ vec![{registry}] }}"),
    )
    .unwrap();
    println!("cargo:rerun-if-changed=schemas");
    protobuf_codegen::Codegen::new()
        .pure()
        .includes(["schemas"])
        .inputs(schemas)
        .cargo_out_dir("generated")
        .run_from_script();
}
