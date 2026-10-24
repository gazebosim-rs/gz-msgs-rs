//! Standalone protobuf inspection tool. Binary bytes only cross stdin/stdout.
use std::io::{Read, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args == ["list"] {
        let mut names: Vec<_> = gz_msgs::files()
            .into_iter()
            .flat_map(|file| {
                file.messages()
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
        println!(
            "{}",
            gz_msgs::protobuf::text_format::print_to_string(descriptor.proto())
        );
        return Ok(());
    }
    let mut input = Vec::new();
    std::io::stdin().read_to_end(&mut input)?;
    let output = match args[0].as_str() {
        "encode-json" => gz_msgs::protobuf_json_mapping::parse_dyn_from_str(
            &descriptor,
            std::str::from_utf8(&input)?,
        )?
        .write_to_bytes_dyn()?,
        "decode-json" => {
            gz_msgs::protobuf_json_mapping::print_to_string(&*descriptor.parse_from_bytes(&input)?)?
                .into_bytes()
        }
        "encode-text" => {
            let mut message = descriptor.new_instance();
            gz_msgs::protobuf::text_format::merge_from_str(
                &mut *message,
                std::str::from_utf8(&input)?,
            )?;
            message.write_to_bytes_dyn()?
        }
        "decode-text" => {
            gz_msgs::protobuf::text_format::print_to_string(&*descriptor.parse_from_bytes(&input)?)
                .into_bytes()
        }
        _ => return Err("unknown command".into()),
    };
    let mut stdout = std::io::stdout().lock();
    stdout.write_all(&output)?;
    stdout.flush()?;
    Ok(())
}
