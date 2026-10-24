// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2012 Open Source Robotics Foundation
// Copyright (C) 2023 Open Source Robotics Foundation
//! Spherical reference parameters, without geodetic transformation algorithms.
use crate::msgs::SphericalCoordinates;
use std::f64::consts::PI;
const EARTH_AXES: [f64; 2] = [6378137., 6356752.314245];
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum SphericalSurface {
    #[default]
    EarthWgs84,
    MoonScs,
    Custom {
        axis_equatorial: f64,
        axis_polar: f64,
    },
    /// Preserve a native math surface code that the converter does not recognize.
    Unrecognized(i32),
}
impl SphericalSurface {
    /// Native model axes in meters; unrecognized models have no represented axes.
    pub fn axes(self) -> Option<[f64; 2]> {
        match self {
            Self::EarthWgs84 => Some(EARTH_AXES),
            Self::MoonScs => Some([1738100., 1736000.]),
            Self::Custom {
                axis_equatorial,
                axis_polar,
            } => Some([axis_equatorial, axis_polar]),
            Self::Unrecognized(_) => None,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SphericalComponents {
    pub surface: SphericalSurface,
    pub latitude_rad: f64,
    pub longitude_rad: f64,
    pub elevation: f64,
    pub heading_rad: f64,
}
/// Create a fresh message from caller-computed surface/reference components.
pub fn spherical_coordinates(value: SphericalComponents) -> SphericalCoordinates {
    let mut message = SphericalCoordinates::default();
    set_spherical_coordinates(&mut message, value);
    message
}
/// Preserve header/entity and unused axes; unknown surfaces leave old model/axes.
/// Radian/degree operation order matches native Angle to preserve rounding.
pub fn set_spherical_coordinates(message: &mut SphericalCoordinates, value: SphericalComponents) {
    match value.surface {
        SphericalSurface::EarthWgs84 => message.surface_model = 0,
        SphericalSurface::MoonScs => message.surface_model = 1,
        SphericalSurface::Custom {
            axis_equatorial,
            axis_polar,
        } => {
            message.surface_model = 2;
            message.surface_axis_equatorial = axis_equatorial;
            message.surface_axis_polar = axis_polar;
        }
        SphericalSurface::Unrecognized(code) => {
            eprintln!("Unrecognized spherical surface type [{code}]. Not populating message field.")
        }
    }
    message.latitude_deg = value.latitude_rad * 180. / PI;
    message.longitude_deg = value.longitude_rad * 180. / PI;
    message.elevation = value.elevation;
    message.heading_deg = value.heading_rad * 180. / PI;
}
/// Read fresh reference components, reproducing native custom-axis fallback.
/// Invalid custom axes retain CUSTOM but use Earth's parameters; unknown models
/// retain the fresh domain default EARTH. Coordinates are not normalized.
pub fn spherical_components(message: &SphericalCoordinates) -> SphericalComponents {
    let surface = match message.surface_model {
        0 => SphericalSurface::EarthWgs84,
        1 => SphericalSurface::MoonScs,
        2 => {
            let mut axes = [message.surface_axis_equatorial, message.surface_axis_polar];
            if !(axes[0] > 0. && axes[1] > 0. && axes[1] <= axes[0]) {
                eprintln!("Invalid parameters found, defaulting to Earth's parameters");
                axes = EARTH_AXES;
            }
            SphericalSurface::Custom {
                axis_equatorial: axes[0],
                axis_polar: axes[1],
            }
        }
        code => {
            eprintln!("Unrecognized spherical surface type [{code}]. Not populating data field.");
            SphericalSurface::EarthWgs84
        }
    };
    SphericalComponents {
        surface,
        latitude_rad: message.latitude_deg * PI / 180.,
        longitude_rad: message.longitude_deg * PI / 180.,
        elevation: message.elevation,
        heading_rad: message.heading_deg * PI / 180.,
    }
}
