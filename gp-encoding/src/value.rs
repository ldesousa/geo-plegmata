// Copyright 2026 contributors to the GeoPlegmata project.
// Originally authored by Francisco Salgueiro, Instituto Superior Técnico (francisco.salgueiro@tecnico.ulisboa.pt)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use serde_json::{Number, Value};

use crate::error::EncodingError;
use crate::models::DataType;

pub fn decode_value_to_json(dtype: &DataType, bytes: &[u8]) -> Result<Value, EncodingError> {
    match dtype {
        DataType::Float32 => {
            let value = f32::from_ne_bytes(parse_fixed::<4>(bytes)?);
            Number::from_f64(value as f64)
                .map(Value::Number)
                .ok_or_else(|| {
                    EncodingError::Storage(format!("cannot represent {value} as JSON number"))
                })
        }
        DataType::Float64 => {
            let value = f64::from_ne_bytes(parse_fixed::<8>(bytes)?);
            Number::from_f64(value).map(Value::Number).ok_or_else(|| {
                EncodingError::Storage(format!("cannot represent {value} as JSON number"))
            })
        }
        DataType::Int8 => Ok(Value::Number(
            i8::from_ne_bytes(parse_fixed::<1>(bytes)?).into(),
        )),
        DataType::Int16 => Ok(Value::Number(
            i16::from_ne_bytes(parse_fixed::<2>(bytes)?).into(),
        )),
        DataType::Int32 => Ok(Value::Number(
            i32::from_ne_bytes(parse_fixed::<4>(bytes)?).into(),
        )),
        DataType::Int64 => Ok(Value::Number(
            i64::from_ne_bytes(parse_fixed::<8>(bytes)?).into(),
        )),
        DataType::UInt8 => Ok(Value::Number(
            u8::from_ne_bytes(parse_fixed::<1>(bytes)?).into(),
        )),
        DataType::UInt16 => Ok(Value::Number(
            u16::from_ne_bytes(parse_fixed::<2>(bytes)?).into(),
        )),
        DataType::UInt32 => Ok(Value::Number(
            u32::from_ne_bytes(parse_fixed::<4>(bytes)?).into(),
        )),
        DataType::UInt64 => Ok(Value::Number(
            u64::from_ne_bytes(parse_fixed::<8>(bytes)?).into(),
        )),
    }
}

pub fn decode_value_to_f64(dtype: &DataType, bytes: &[u8]) -> Result<f64, EncodingError> {
    match dtype {
        DataType::Float32 => Ok(f32::from_ne_bytes(parse_fixed::<4>(bytes)?) as f64),
        DataType::Float64 => Ok(f64::from_ne_bytes(parse_fixed::<8>(bytes)?)),
        DataType::Int8 => Ok(i8::from_ne_bytes(parse_fixed::<1>(bytes)?) as f64),
        DataType::Int16 => Ok(i16::from_ne_bytes(parse_fixed::<2>(bytes)?) as f64),
        DataType::Int32 => Ok(i32::from_ne_bytes(parse_fixed::<4>(bytes)?) as f64),
        DataType::Int64 => Ok(i64::from_ne_bytes(parse_fixed::<8>(bytes)?) as f64),
        DataType::UInt8 => Ok(u8::from_ne_bytes(parse_fixed::<1>(bytes)?) as f64),
        DataType::UInt16 => Ok(u16::from_ne_bytes(parse_fixed::<2>(bytes)?) as f64),
        DataType::UInt32 => Ok(u32::from_ne_bytes(parse_fixed::<4>(bytes)?) as f64),
        DataType::UInt64 => Ok(u64::from_ne_bytes(parse_fixed::<8>(bytes)?) as f64),
    }
}

pub fn parse_fill_value_to_f64(dtype: &DataType, fill_value: &str) -> Result<f64, EncodingError> {
    let trimmed = fill_value.trim();
    match dtype {
        DataType::Float32 | DataType::Float64 => trimmed
            .parse::<f64>()
            .map_err(|e| EncodingError::Storage(format!("invalid fill value '{fill_value}': {e}"))),
        DataType::Int8 => {
            parse_i64_in_range(trimmed, i8::MIN as i64, i8::MAX as i64).map(|value| value as f64)
        }
        DataType::Int16 => {
            parse_i64_in_range(trimmed, i16::MIN as i64, i16::MAX as i64).map(|value| value as f64)
        }
        DataType::Int32 => {
            parse_i64_in_range(trimmed, i32::MIN as i64, i32::MAX as i64).map(|value| value as f64)
        }
        DataType::Int64 => {
            parse_i64_in_range(trimmed, i64::MIN, i64::MAX).map(|value| value as f64)
        }
        DataType::UInt8 => parse_u64_in_range(trimmed, u8::MAX as u64).map(|value| value as f64),
        DataType::UInt16 => parse_u64_in_range(trimmed, u16::MAX as u64).map(|value| value as f64),
        DataType::UInt32 => parse_u64_in_range(trimmed, u32::MAX as u64).map(|value| value as f64),
        DataType::UInt64 => parse_u64_in_range(trimmed, u64::MAX).map(|value| value as f64),
    }
}

pub fn encode_value_from_f64(dtype: &DataType, value: f64) -> Result<Vec<u8>, EncodingError> {
    let bytes = match dtype {
        DataType::Float32 => (value as f32).to_ne_bytes().to_vec(),
        DataType::Float64 => value.to_ne_bytes().to_vec(),
        DataType::Int8 => (clamp_i64(value, i8::MIN as i64, i8::MAX as i64) as i8)
            .to_ne_bytes()
            .to_vec(),
        DataType::Int16 => (clamp_i64(value, i16::MIN as i64, i16::MAX as i64) as i16)
            .to_ne_bytes()
            .to_vec(),
        DataType::Int32 => (clamp_i64(value, i32::MIN as i64, i32::MAX as i64) as i32)
            .to_ne_bytes()
            .to_vec(),
        DataType::Int64 => clamp_i64(value, i64::MIN, i64::MAX).to_ne_bytes().to_vec(),
        DataType::UInt8 => (clamp_u64(value, u8::MAX as u64) as u8)
            .to_ne_bytes()
            .to_vec(),
        DataType::UInt16 => (clamp_u64(value, u16::MAX as u64) as u16)
            .to_ne_bytes()
            .to_vec(),
        DataType::UInt32 => (clamp_u64(value, u32::MAX as u64) as u32)
            .to_ne_bytes()
            .to_vec(),
        DataType::UInt64 => clamp_u64(value, u64::MAX).to_ne_bytes().to_vec(),
    };
    Ok(bytes)
}


pub fn format_value(dtype: &DataType, bytes: &[u8]) -> Result<String, EncodingError> {
    decode_value_to_json(dtype, bytes).map(|value| value.to_string())
}

fn parse_fixed<const N: usize>(bytes: &[u8]) -> Result<[u8; N], EncodingError> {
    if bytes.len() < N {
        return Err(EncodingError::Storage(format!(
            "not enough bytes to decode value: expected at least {N}, got {}",
            bytes.len()
        )));
    }

    let mut arr = [0_u8; N];
    arr.copy_from_slice(&bytes[..N]);
    Ok(arr)
}

fn parse_i64_in_range(value: &str, min: i64, max: i64) -> Result<i64, EncodingError> {
    let parsed = match value.parse::<i64>() {
        Ok(parsed) => parsed,
        Err(_) => {
            let float_value = value.parse::<f64>().map_err(|e| {
                EncodingError::Storage(format!("invalid fill value '{value}': {e}"))
            })?;
            if !float_value.is_finite() || float_value.fract() != 0.0 {
                return Err(EncodingError::Storage(format!(
                    "fill value {float_value} is not a valid integer"
                )));
            }
            float_value as i64
        }
    };
    if !(min..=max).contains(&parsed) {
        return Err(EncodingError::Storage(format!(
            "fill value {parsed} is outside range [{min}, {max}]"
        )));
    }
    Ok(parsed)
}

fn parse_u64_in_range(value: &str, max: u64) -> Result<u64, EncodingError> {
    let parsed = match value.parse::<u64>() {
        Ok(parsed) => parsed,
        Err(_) => {
            let float_value = value.parse::<f64>().map_err(|e| {
                EncodingError::Storage(format!("invalid fill value '{value}': {e}"))
            })?;
            if !float_value.is_finite() || float_value.fract() != 0.0 || float_value < 0.0 {
                return Err(EncodingError::Storage(format!(
                    "fill value {float_value} is not a valid unsigned integer"
                )));
            }
            float_value as u64
        }
    };
    if parsed > max {
        return Err(EncodingError::Storage(format!(
            "fill value {parsed} is outside range [0, {max}]"
        )));
    }
    Ok(parsed)
}

fn clamp_i64(value: f64, min: i64, max: i64) -> i64 {
    if !value.is_finite() {
        return 0;
    }
    let rounded = value.round();
    let clamped = rounded.max(min as f64).min(max as f64);
    clamped as i64
}

fn clamp_u64(value: f64, max: u64) -> u64 {
    if !value.is_finite() {
        return 0;
    }
    let rounded = value.round();
    let clamped = rounded.max(0.0).min(max as f64);
    clamped as u64
}
