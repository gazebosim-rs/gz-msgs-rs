use heck::{ToSnakeCase, ToUpperCamelCase};
use prost::Message;
fn main() {
    let mut schemas: Vec<_> = std::fs::read_dir("schemas/gz/msgs")
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "proto"))
        .collect();
    schemas.sort();
    println!("cargo:rerun-if-changed=schemas");
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    prost_build::Config::new()
        .prost_path("crate::runtime")
        .enable_type_names()
        .type_name_domain(["."], "type.googleapis.com")
        .file_descriptor_set_path(out.join("messages.pb"))
        .compile_protos(&schemas, &["schemas"])
        .unwrap();
    let descriptors = prost_types::FileDescriptorSet::decode(
        std::fs::read(out.join("messages.pb")).unwrap().as_slice(),
    )
    .unwrap();
    let mut aliases = String::new();
    for file in descriptors
        .file
        .iter()
        .filter(|f| f.name().starts_with("gz/msgs/"))
    {
        let stem = std::path::Path::new(file.name())
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned();
        aliases += &format!("pub mod {stem} {{");
        for message in &file.message_type {
            aliases += &format!(
                "pub use crate::msgs::{} as {};",
                message.name().to_upper_camel_case(),
                message.name()
            );
            if !message.enum_type.is_empty()
                || !message.oneof_decl.is_empty()
                || message
                    .nested_type
                    .iter()
                    .any(|m| !m.options.as_ref().is_some_and(|o| o.map_entry()))
            {
                let nested = message.name().to_snake_case();
                aliases += &format!("pub use crate::msgs::{nested};");
            }
        }
        for enumeration in &file.enum_type {
            aliases += &format!(
                "pub use crate::msgs::{} as {};",
                enumeration.name().to_upper_camel_case(),
                enumeration.name()
            );
        }
        aliases += "}";
    }
    std::fs::write(out.join("aliases.rs"), aliases).unwrap();
    // SourceCodeInfo supplies exact declaration spans; no .proto parser is needed.
    let mut definitions = String::from(
        "pub fn message_definition(name: &str) -> Option<&'static str> { match name {",
    );
    for file in descriptors
        .file
        .iter()
        .filter(|f| f.name().starts_with("gz/msgs/"))
    {
        let source =
            std::fs::read_to_string(std::path::Path::new("schemas").join(file.name())).unwrap();
        for location in &file
            .source_code_info
            .as_ref()
            .expect("protoc source info")
            .location
        {
            let path = &location.path;
            if path.len() < 2
                || path.len() % 2 != 0
                || path[0] != 4
                || path.iter().skip(2).step_by(2).any(|part| *part != 3)
            {
                continue;
            }
            let mut message = &file.message_type[path[1] as usize];
            let mut name = format!("{}.{}", file.package(), message.name());
            for index in path[2..].chunks_exact(2) {
                message = &message.nested_type[index[1] as usize];
                name += &format!(".{}", message.name());
            }
            let span = &location.span;
            assert_eq!(span.len(), 4, "pinned canonical message source span");
            let offset = |line: i32, column: i32| {
                source
                    .split_inclusive('\n')
                    .take(line as usize)
                    .map(str::len)
                    .sum::<usize>()
                    + column as usize
            };
            let definition = &source[offset(span[0], span[1])..offset(span[2], span[3])];
            let brace = definition.find('{').expect("message declaration body");
            let text = format!("message {} {}", message.name(), &definition[brace..]);
            definitions += &format!("{name:?} => Some({text:?}),");
        }
    }
    definitions += "_ => None } }";
    std::fs::write(out.join("definitions.rs"), definitions).unwrap();
}
