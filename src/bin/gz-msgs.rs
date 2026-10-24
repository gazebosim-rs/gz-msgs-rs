//! Standalone protobuf inspection tool. Binary bytes only cross stdin/stdout.
use gz_msgs::{prost::Message, prost_reflect::DynamicMessage};
use std::io::{Read, Write};
const HELP: &str = "gz-msgs --list|-l | --info|-i TYPE... | --version|-v | --help|-h\nCommands: list, info TYPE..., encode-json, decode-json, encode-text, decode-text\nFormat commands take one fully qualified type and use stdin/stdout.";
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.is_empty() || args == ["--help"] || args == ["-h"] {
        println!("{HELP}");
        return Ok(());
    }
    if args == ["--version"] || args == ["-v"] {
        println!("{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    if args == ["list"] || args == ["--list"] || args == ["-l"] {
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
    if args.len() >= 2 && ["info", "--info", "-i"].contains(&args[0].as_str()) {
        for name in &args[1..] {
            let descriptor = gz_msgs::descriptor(name).ok_or("unknown message type")?;
            let text = gz_msgs::message_info(descriptor.full_name())
                .ok_or("message info is not bundled")?;
            print!("{text}");
        }
        return Ok(());
    }
    if args.len() != 2 {
        return Err(HELP.into());
    }
    let descriptor = gz_msgs::descriptor(&args[1]).ok_or("unknown message type")?;
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
