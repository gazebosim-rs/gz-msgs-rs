// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
//! Legacy Gazebo enum spelling/fallback policy layered over canonical descriptors.
use crate::prost_reflect::EnumDescriptor;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnumKind {
    Joint,
    Geometry,
    Shader,
    PixelFormat,
    Discovery,
}
impl EnumKind {
    fn config(self) -> (&'static str, bool, &'static str, &'static str) {
        match self {
            Self::Joint => (
                "gz.msgs.Joint.Type",
                true,
                "JointType",
                "msgs::Joint::REVOLUTE",
            ),
            Self::Geometry => (
                "gz.msgs.Geometry.Type",
                true,
                "Geometry::Type",
                "msgs::Geometry::BOX",
            ),
            Self::Shader => (
                "gz.msgs.Material.ShaderType",
                true,
                "Material::ShaderType",
                "msgs::Material::VERTEX",
            ),
            Self::PixelFormat => (
                "gz.msgs.PixelFormatType",
                false,
                "PixelFormatType",
                "msgs::PixelFormatType::UNKNOWN_PIXEL_FORMAT",
            ),
            Self::Discovery => (
                "gz.msgs.Discovery.Type",
                false,
                "DiscoveryType",
                "msgs::Discovery::UNINITIALIZED",
            ),
        }
    }
    fn descriptor(self) -> EnumDescriptor {
        crate::descriptors()
            .get_enum_by_name(self.config().0)
            .expect("canonical enum")
    }
    fn supported(self, name: &str) -> bool {
        // Geometry's legacy conversion deliberately omits several valid wire enums.
        self != Self::Geometry
            || matches!(
                name,
                "BOX"
                    | "CAPSULE"
                    | "CONE"
                    | "CYLINDER"
                    | "ELLIPSOID"
                    | "SPHERE"
                    | "PLANE"
                    | "IMAGE"
                    | "HEIGHTMAP"
                    | "MESH"
                    | "POLYLINE"
            )
    }
    /// Case-sensitive legacy spelling; unknown input logs/returns the native fallback.
    pub fn parse(self, name: &str) -> i32 {
        let (_, lower, label, fallback) = self.config();
        let wire = if lower {
            name.to_ascii_uppercase()
        } else {
            name.to_owned()
        };
        let candidate = self.descriptor().get_value_by_name(&wire).filter(|value| {
            self.supported(value.name())
                && if lower {
                    value.name().to_ascii_lowercase() == name
                } else {
                    value.name() == name
                }
        });
        if let Some(value) = candidate {
            return value.number();
        }
        match self {
            Self::PixelFormat => {}
            Self::Discovery => {
                eprintln!("Unrecognized DiscoveryType[{name}], returning {fallback}")
            }
            _ => eprintln!("Unrecognized {label} [{name}], returning {fallback}"),
        }
        0
    }
    /// Format a raw enum integer, retaining native unknown-value fallbacks.
    pub fn format(self, number: i32) -> String {
        let (_, lower, label, _) = self.config();
        let descriptor = self.descriptor();
        if let Some(value) = descriptor
            .get_value(number)
            .filter(|value| self.supported(value.name()))
        {
            if lower {
                value.name().to_ascii_lowercase()
            } else {
                value.name().to_owned()
            }
        } else if lower {
            eprintln!("Unrecognized {label} [{number}], returning 'unknown'");
            "unknown".into()
        } else {
            descriptor
                .get_value(0)
                .expect("canonical enum zero")
                .name()
                .to_owned()
        }
    }
}
