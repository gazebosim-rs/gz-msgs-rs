// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2016 Open Source Robotics Foundation
use gz_msgs::{fuel, msgs::FuelMetadata, runtime::Message};
fn bytes(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}
#[test]
fn native_fuel_metadata_wire_and_xml_corpus() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/fuel-reference.json")).unwrap();
    assert_eq!(fixture["rows"].as_array().unwrap().len(), 173);
    for row in fixture["rows"].as_array().unwrap() {
        let parsed = fuel::from_xml(row["xml"].as_str().unwrap());
        if row.get("exception").is_some() {
            assert_eq!(parsed, Err(fuel::Error::InvalidVersion), "{row}");
        } else if row["parsed"] == true {
            let parsed = parsed.unwrap_or_else(|e| panic!("{e:?} {row}"));
            let wire = bytes(row["wire_hex"].as_str().unwrap());
            assert_eq!(
                parsed,
                FuelMetadata::decode(wire.as_slice()).unwrap(),
                "{row}"
            );
            assert_eq!(parsed.encode_to_vec(), wire, "{row}");
            assert_eq!(
                fuel::to_xml(&parsed).unwrap().as_bytes(),
                bytes(row["xml_hex"].as_str().unwrap()),
                "{row}"
            );
        } else {
            assert!(parsed.is_err(), "{row}");
        }
    }
}

#[test]
fn original_convert_fuel_metadata() {
    use gz_msgs::msgs::{
        fuel_metadata::{Contact, Dependency, Model, ResourceType, World},
        Version, VersionedName,
    };
    for xml in [
        "",
        "<test/>",
        "<model>test</model>",
        "<model><name>test_model</name></model>",
    ] {
        assert!(fuel::from_xml(xml).is_err());
    }
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/fuel-reference.json")).unwrap();
    let rows = fixture["rows"].as_array().unwrap();
    let model_xml = rows.iter().find(|r| r["id"] == "original_model").unwrap()["xml"]
        .as_str()
        .unwrap();
    let world_xml = rows.iter().find(|r| r["id"] == "original_world").unwrap()["xml"]
        .as_str()
        .unwrap();
    let model = fuel::from_xml(model_xml).unwrap();
    assert_eq!(model.name, "test_model");
    assert_eq!(model.version, 3);
    assert_eq!(model.description, "A model for testing");
    let Some(ResourceType::Model(resource)) = &model.resource_type else {
        panic!("model expected")
    };
    assert_eq!(resource.file, "model.sdf");
    let format = resource.file_format.as_ref().unwrap();
    assert_eq!(format.name, "sdf");
    assert_eq!(format.version.as_ref().unwrap().major, 1);
    assert_eq!(format.version.as_ref().unwrap().minor, 7);
    assert_eq!(model.authors.len(), 1);
    assert_eq!(model.authors[0].name, "Foo Bar");
    assert_eq!(model.authors[0].email, "foo@bar.org");
    assert_eq!(model.dependencies.len(), 1);
    assert_eq!(model.dependencies[0].uri, "model://some_model");
    let world = fuel::from_xml(world_xml).unwrap();
    assert_eq!(world.name, "test_world");
    assert_eq!(world.version, 2);
    assert_eq!(world.description, "A world for testing");
    let Some(ResourceType::World(resource)) = &world.resource_type else {
        panic!("world expected")
    };
    assert_eq!(resource.file, "world.sdf");
    let format = resource.file_format.as_ref().unwrap();
    assert_eq!(format.name, "sdf");
    assert_eq!(format.version.as_ref().unwrap().major, 1);
    assert_eq!(format.version.as_ref().unwrap().minor, 7);
    assert_eq!(world.authors.len(), 0);
    assert_eq!(world.dependencies.len(), 0);
    let mut output = FuelMetadata {
        resource_type: Some(ResourceType::World(World {
            file: "world.sdf".into(),
            ..World::default()
        })),
        ..FuelMetadata::default()
    };
    assert_eq!(fuel::to_xml(&output), Err(fuel::Error::NotSdf));
    output.name = "test_world".into();
    output.version = 2;
    output.description = "A world for testing".into();
    let format = Some(VersionedName {
        name: "sdf".into(),
        version: Some(Version {
            major: 1,
            minor: 7,
            ..Version::default()
        }),
    });
    let Some(ResourceType::World(resource)) = &mut output.resource_type else {
        unreachable!()
    };
    resource.file_format = format.clone();
    assert_eq!(fuel::to_xml(&output).unwrap(), world_xml);
    output = FuelMetadata {
        resource_type: Some(ResourceType::Model(Model {
            file: "model.sdf".into(),
            ..Model::default()
        })),
        ..FuelMetadata::default()
    };
    assert_eq!(fuel::to_xml(&output), Err(fuel::Error::NotSdf));
    output.name = "test_model".into();
    output.version = 3;
    output.description = "A model for testing".into();
    let Some(ResourceType::Model(resource)) = &mut output.resource_type else {
        unreachable!()
    };
    resource.file_format = format;
    assert!(fuel::to_xml(&output).is_ok());
    output.authors.push(Contact {
        name: "Foo Bar".into(),
        email: "foo@bar.org".into(),
    });
    output.dependencies.push(Dependency {
        uri: "model://some_model".into(),
    });
    assert_eq!(fuel::to_xml(&output).unwrap(), model_xml);
}

#[test]
fn safe_missing_uri_and_serializer_defaults() {
    use gz_msgs::msgs::{
        fuel_metadata::{Model, ResourceType},
        VersionedName,
    };
    assert_eq!(fuel::from_xml("<model><name>n</name><sdf version='1.7'>m</sdf><depend><model><uri/></model></depend></model>"), Err(fuel::Error::MissingUri));
    assert_eq!(
        fuel::to_xml(&FuelMetadata::default()),
        Err(fuel::Error::NotSdf)
    );
    let mut meta = FuelMetadata {
        resource_type: Some(ResourceType::Model(Model {
            file: "m<&".into(),
            file_format: Some(VersionedName {
                name: "sdf".into(),
                version: None,
            }),
        })),
        ..FuelMetadata::default()
    };
    let xml = fuel::to_xml(&meta).unwrap();
    assert!(xml.contains("<sdf version='0.0'>m<&</sdf>"));
    assert!(fuel::from_xml(&xml).is_err());
    let Some(ResourceType::Model(model)) = &mut meta.resource_type else {
        unreachable!()
    };
    model.file_format.as_mut().unwrap().name = "other".into();
    assert_eq!(fuel::to_xml(&meta), Err(fuel::Error::NotSdf));
}

#[test]
fn invalid_unicode_references_have_no_rust_string_representation() {
    for entity in [
        "&#xD800;",
        "&#55296;",
        "&#x110000;",
        "&#999999999999999999999999999999;",
    ] {
        assert_eq!(
            fuel::from_xml(&format!(
                "<model><name>{entity}</name><sdf version='1.7'>m</sdf></model>"
            )),
            Err(fuel::Error::Xml)
        );
        assert_eq!(
            fuel::from_xml(&format!(
                "<model><name>n</name><sdf version='1.7{entity}'>m</sdf></model>"
            )),
            Err(fuel::Error::Xml)
        );
    }
}
