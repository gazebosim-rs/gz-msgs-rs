//! Standalone protobuf inspection tool. Binary bytes only cross stdin/stdout.
use gz_msgs::{prost::Message, prost_reflect::DynamicMessage};
use std::io::{Read, Write};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args == ["list"] {
        let mut names: Vec<_> = gz_msgs::files()
            .into_iter()
            .flat_map(|f| {
                f.messages()
                    .map(|m| m.full_name().to_owned())
                    .collect::<Vec<_>>()
            })
            .collect();
        names.sort();
        println!("{}", names.join("\n"));
        return Ok(());
    }
    if args.len() != 2 {
        return Err("usage: gz-msgs list | info|encode-json|decode-json|encode-text|decode-text gz.msgs.Type".into());
    }
    let descriptor = gz_msgs::descriptor(&args[1]).ok_or("unknown message type")?;
    if args[0] == "info" {
        println!("{:#?}", descriptor.descriptor_proto());
        return Ok(());
    }
    let mut input = Vec::new();
    std::io::stdin().read_to_end(&mut input)?;
    let output = match args[0].as_str() {
        "encode-json" => DynamicMessage::deserialize(
            descriptor,
            &mut gz_msgs::serde_json::Deserializer::from_slice(&input),
        )?
        .encode_to_vec(),
        "decode-json" => {
            gz_msgs::serde_json::to_vec(&DynamicMessage::decode(descriptor, input.as_slice())?)?
        }
        "encode-text" => {
            DynamicMessage::parse_text_format(descriptor, std::str::from_utf8(&input)?)?
                .encode_to_vec()
        }
        "decode-text" => DynamicMessage::decode(descriptor, input.as_slice())?
            .to_text_format()
            .into_bytes(),
        _ => return Err("unknown command".into()),
    };
    let mut stdout = std::io::stdout().lock();
    stdout.write_all(&output)?;
    stdout.flush()?;
    Ok(())
}
