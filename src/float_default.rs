// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
//! Pinned protobuf declaration policy; Rust std owns digit generation/rounding.

fn trim_fraction(value: &str) -> &str {
    if value.contains('.') {
        value.trim_end_matches('0').trim_end_matches('.')
    } else {
        value
    }
}

pub(crate) fn float(value: f32) -> String {
    render(f64::from(value), Some(value))
}

pub(crate) fn double(value: f64) -> String {
    render(value, None)
}

fn render(value: f64, single: Option<f32>) -> String {
    if value.is_nan() {
        return "nan".into();
    }
    if value.is_infinite() {
        return value.to_string();
    }
    let mut digits = if single.is_some() { 6 } else { 15 };
    let mut scientific = format!("{value:.precision$e}", precision = digits - 1);
    let roundtrips = if let Some(original) = single {
        // Native safe_strtof treats inexact subnormal conversion as ERANGE.
        !original.is_subnormal()
            && scientific.parse::<f32>().expect("std decimal float") == original
    } else {
        scientific.parse::<f64>().expect("std decimal double") == value
    };
    if !roundtrips {
        digits = if single.is_some() { 9 } else { 17 };
        scientific = format!("{value:.precision$e}", precision = digits - 1);
    }
    let (mantissa, exponent) = scientific.split_once('e').expect("std scientific format");
    let exponent: i32 = exponent.parse().expect("std decimal exponent");
    // Protobuf's %g chooses notation after rounding and pads exponent signs.
    if exponent < -4 || exponent >= digits as i32 {
        format!("{}e{exponent:+03}", trim_fraction(mantissa))
    } else {
        let places = (digits as i32 - 1 - exponent) as usize;
        let fixed = format!("{value:.places$}");
        trim_fraction(&fixed).into()
    }
}

#[cfg(test)]
#[path = "../tests/private/float_default.rs"]
mod tests;
