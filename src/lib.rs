#![doc = include_str!("../README.md")]

extern crate num_traits;
use num_traits::{Bounded, Float, ToPrimitive};
use std::error::Error;
use std::io::ErrorKind;
use std::io::{Read, Write};

/// The Sample trait is used for low-level conversions of samples stored as raw bytes, to f32 or f64 sample values.
///
/// The float values are expected to use the range -1.0 <= value < +1.0.
/// The integer types are mapped to this range.
/// Using f32, up to 24 byte integers can be converted without loss to and from float.
/// 32-bit integers required the use of f64 for lossless conversion.
///
/// The exact range depends on the format. The lower limit is always -1.0. But the upper limit is (2^(n-1)-1)/2^(n-1).
/// For example for 16-bit integer, the maximum value is (2^15-1)/2^15, approximately +0.99997.
///
/// When reading from raw bytes, there is no clipping. Integer types cannot go outside the range.
/// Float values are read as they are, and are thus allowed to be outside the -1.0 to +1.0 range.
///
/// When writing samples, the float sample values are clamped to the range supported by the chosen format.
/// Float output values are also clamped to the -1.0 to +1.0 range, since this is what most audio APIs expect.

pub trait Sample<T: Sized> {
    const MAX_I32: T;
    const MAX_I24: T;
    const MAX_I16: T;

    /// Convert a sample value to S32LE (4 bytes)
    fn to_s32_le(&self) -> ([u8; 4], bool);
    /// Convert a sample value to S32BE (4 bytes)
    fn to_s32_be(&self) -> ([u8; 4], bool);
    /// Convert a sample value to S24LE3 (3 bytes)
    fn to_s24_3_le(&self) -> ([u8; 3], bool);
    /// Convert a sample value to S24BE3 (3 bytes)
    fn to_s24_3_be(&self) -> ([u8; 3], bool);
    /// Convert a sample value to S24LE4 (4 bytes)
    fn to_s24_4_le(&self) -> ([u8; 4], bool);
    /// Convert a sample value to S24BE4 (4 bytes)
    fn to_s24_4_be(&self) -> ([u8; 4], bool);
    /// Convert a sample value to S16LE (2 bytes)
    fn to_s16_le(&self) -> ([u8; 2], bool);
    /// Convert a sample value to S16BE (2 bytes)
    fn to_s16_be(&self) -> ([u8; 2], bool);
    /// Convert a sample value to F64LE (8 bytes)
    fn to_f64_le(&self) -> ([u8; 8], bool);
    /// Convert a sample value to F64BE (8 bytes)
    fn to_f64_be(&self) -> ([u8; 8], bool);
    /// Convert a sample value to F32LE (4 bytes)
    fn to_f32_le(&self) -> ([u8; 4], bool);
    /// Convert a sample value to F32BE (4 bytes)
    fn to_f32_be(&self) -> ([u8; 4], bool);

    /// Convert S32LE (4 bytes) to a sample value
    fn from_s32_le(bytes: [u8; 4]) -> Self;
    /// Convert S32BE (4 bytes) to a sample value
    fn from_s32_be(bytes: [u8; 4]) -> Self;
    /// Convert S16LE (2 bytes) to a sample value
    fn from_s16_le(bytes: [u8; 2]) -> Self;
    /// Convert S16BE (2 bytes) to a sample value
    fn from_s16_be(bytes: [u8; 2]) -> Self;
    /// Convert S24LE3 (3 bytes) to a sample value
    fn from_s24_3_le(bytes: [u8; 3]) -> Self;
    /// Convert S24BE3 (3 bytes) to a sample value
    fn from_s24_3_be(bytes: [u8; 3]) -> Self;
    /// Convert S24LE4 (4 bytes) to a sample value
    fn from_s24_4_le(bytes: [u8; 4]) -> Self;
    /// Convert S24BE4 (4 bytes) to a sample value
    fn from_s24_4_be(bytes: [u8; 4]) -> Self;
    /// Convert F32LE (4 bytes) to a sample value
    fn from_f32_le(bytes: [u8; 4]) -> Self;
    /// Convert F32BE (4 bytes) to a sample value
    fn from_f32_be(bytes: [u8; 4]) -> Self;
    /// Convert F64LE (8 bytes) to a sample value
    fn from_f64_le(bytes: [u8; 8]) -> Self;
    /// Convert F64BE (8 bytes) to a sample value
    fn from_f64_be(bytes: [u8; 8]) -> Self;
}

/// The supported sample formats.
pub enum SampleFormat {
    /// 16 bit signed integer, little endian.
    S16LE,
    /// 16 bit signed integer, big endian.
    S16BE,
    /// 24 bit signed integer, little endian, 24 bytes stored as 3 bytes.
    S24LE3,
    /// 24 bit signed integer, big endian, 24 bytes stored as 3 bytes.
    S24BE3,
    /// 24 bit signed integer, little endian, stored as 4 bytes. The data is in the lower 3 bytes and the most significant byte is padding.
    S24LE4,
    /// 24 bit signed integer, big endian, stored as 4 bytes. The data is in the lower 3 bytes and the most significant byte is padding.
    S24BE4,
    /// 32 bit signed integer, little endian.
    S32LE,
    /// 32 bit signed integer, big endian.
    S32BE,
    /// 32 bit floating point, little endian.
    F32LE,
    /// 32 bit floating point, big endian.
    F32BE,
    /// 64 bit floating point, little endian.
    F64LE,
    /// 64 bit floating point, big endian.
    F64BE,
}

impl SampleFormat {
    /// Get the number of bytes that the format uses to store each sample.
    pub fn bytes_per_sample(&self) -> usize {
        match self {
            SampleFormat::S16LE => 2,
            SampleFormat::S16BE => 2,
            SampleFormat::S24LE3 => 3,
            SampleFormat::S24BE3 => 3,
            SampleFormat::S24LE4 => 4,
            SampleFormat::S24BE4 => 4,
            SampleFormat::S32LE => 4,
            SampleFormat::S32BE => 4,
            SampleFormat::F32LE => 4,
            SampleFormat::F32BE => 4,
            SampleFormat::F64LE => 8,
            SampleFormat::F64BE => 8,
        }
    }
}

macro_rules! write_samples {
    ($values:expr, $target:expr, $conv:ident) => {{
        let mut nbr_clipped = 0;
        for value in $values.iter() {
            let (bytes, clipped) = value.$conv();
            if clipped {
                nbr_clipped += 1;
            }
            $target.write_all(&bytes)?;
        }
        nbr_clipped
    }};
}

/// The SampleWriter trait enables converting and writing many sample values from a slice.
pub trait SampleWriter<T: Sample<T>> {
    /// Write sample values from a slice to anything that implements the "Write" trait.
    /// This can be for example a file, or a Vec of u8.
    /// Input samples are f32 or f64, and are converted to the given sample format.
    /// The sample values are clamped to the range supported by the output format.
    /// For the float types, the input range is -1.0 to +1.0.
    /// For the integer types, the input range doesn't include 1.0.
    /// For example for I16 the maximum value is (2^15-1)/2^15, approximately +0.99997.
    /// The number of clipped samples is returned.
    fn write_samples(
        values: &[T],
        target: &mut dyn Write,
        sformat: &SampleFormat,
    ) -> Result<usize, Box<dyn Error>> {
        let nbr_clipped = match sformat {
            SampleFormat::S16LE => {
                write_samples!(values, target, to_s16_le)
            }
            SampleFormat::S16BE => {
                write_samples!(values, target, to_s16_be)
            }
            SampleFormat::S24LE3 => {
                write_samples!(values, target, to_s24_3_le)
            }
            SampleFormat::S24BE3 => {
                write_samples!(values, target, to_s24_3_be)
            }
            SampleFormat::S24LE4 => {
                write_samples!(values, target, to_s24_4_le)
            }
            SampleFormat::S24BE4 => {
                write_samples!(values, target, to_s24_4_be)
            }
            SampleFormat::S32LE => {
                write_samples!(values, target, to_s32_le)
            }
            SampleFormat::S32BE => {
                write_samples!(values, target, to_s32_be)
            }
            SampleFormat::F32LE => {
                write_samples!(values, target, to_f32_le)
            }
            SampleFormat::F32BE => {
                write_samples!(values, target, to_f32_be)
            }
            SampleFormat::F64LE => {
                write_samples!(values, target, to_f64_le)
            }
            SampleFormat::F64BE => {
                write_samples!(values, target, to_f64_be)
            }
        };
        Ok(nbr_clipped)
    }
}

impl SampleWriter<f64> for f64 {}
impl SampleWriter<f32> for f32 {}

macro_rules! read_samples_to_slice {
    ($data:expr, $values:expr, $conv:ident, $n:expr) => {{
        let mut nbr_read = 0;
        for value in $values.iter_mut() {
            let mut bytes = [0; $n];
            match $data.read_exact(&mut bytes) {
                Ok(()) => {}
                Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => {
                    break;
                }
                Err(err) => return Err(Box::new(err)),
            }
            let newvalue = T::$conv(bytes);
            *value = newvalue;
            nbr_read += 1;
        }
        nbr_read
    }};
}

macro_rules! read_all_samples_to_vec {
    ($data:expr, $values:expr, $conv:ident, $n:expr) => {{
        let mut bytes = [0; $n];
        loop {
            match $data.read_exact(&mut bytes) {
                Ok(()) => {}
                Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => {
                    break;
                }
                Err(err) => return Err(Box::new(err)),
            }
            let newvalue = T::$conv(bytes);
            $values.push(newvalue);
        }
    }};
}

/// The SampleReader trait enables reading and converting raw bytes and to multiple samples.

pub trait SampleReader<T: Sample<T>> {
    /// Read bytes from anything that implements the "Read" trait.
    /// This can be for example a file, or a slice of u8.
    /// The bytes are then converted to f32 or f64 values, and stored in a slice.
    /// It will read until the samples slice is filled.
    /// If end-of-file of the source is reached before the slice is filled, the remaining values of the slice are left untouched.
    /// The number of samples read is returned.
    fn read_samples(
        rawbytes: &mut dyn Read,
        samples: &mut [T],
        sampleformat: &SampleFormat,
    ) -> Result<usize, Box<dyn Error>> {
        let nbr_read = match sampleformat {
            SampleFormat::S16LE => {
                read_samples_to_slice!(rawbytes, samples, from_s16_le, 2)
            }
            SampleFormat::S16BE => {
                read_samples_to_slice!(rawbytes, samples, from_s16_be, 2)
            }
            SampleFormat::S24LE3 => {
                read_samples_to_slice!(rawbytes, samples, from_s24_3_le, 3)
            }
            SampleFormat::S24BE3 => {
                read_samples_to_slice!(rawbytes, samples, from_s24_3_be, 3)
            }
            SampleFormat::S24LE4 => {
                read_samples_to_slice!(rawbytes, samples, from_s24_4_le, 4)
            }
            SampleFormat::S24BE4 => {
                read_samples_to_slice!(rawbytes, samples, from_s24_4_be, 4)
            }
            SampleFormat::S32LE => {
                read_samples_to_slice!(rawbytes, samples, from_s32_le, 4)
            }
            SampleFormat::S32BE => {
                read_samples_to_slice!(rawbytes, samples, from_s32_be, 4)
            }
            SampleFormat::F32LE => {
                read_samples_to_slice!(rawbytes, samples, from_f32_le, 4)
            }
            SampleFormat::F32BE => {
                read_samples_to_slice!(rawbytes, samples, from_f32_be, 4)
            }
            SampleFormat::F64LE => {
                read_samples_to_slice!(rawbytes, samples, from_f64_le, 8)
            }
            SampleFormat::F64BE => {
                read_samples_to_slice!(rawbytes, samples, from_f64_be, 8)
            }
        };
        Ok(nbr_read)
    }

    /// Read all bytes from anything that implements the "Read" trait.
    /// This can be for example a file, or a slice of u8.
    /// The bytes are then converted to f32 or f64 values, and appended to a vec.
    /// It will continue reading until reaching end-of-file of the source.
    /// The number of samples read is returned.
    fn read_all_samples(
        rawbytes: &mut dyn Read,
        samples: &mut Vec<T>,
        sampleformat: &SampleFormat,
    ) -> Result<usize, Box<dyn Error>> {
        let start_len = samples.len();
        match sampleformat {
            SampleFormat::S16LE => {
                read_all_samples_to_vec!(rawbytes, samples, from_s16_le, 2);
            }
            SampleFormat::S16BE => {
                read_all_samples_to_vec!(rawbytes, samples, from_s16_be, 2);
            }
            SampleFormat::S24LE3 => {
                read_all_samples_to_vec!(rawbytes, samples, from_s24_3_le, 3);
            }
            SampleFormat::S24BE3 => {
                read_all_samples_to_vec!(rawbytes, samples, from_s24_3_be, 3);
            }
            SampleFormat::S24LE4 => {
                read_all_samples_to_vec!(rawbytes, samples, from_s24_4_le, 4);
            }
            SampleFormat::S24BE4 => {
                read_all_samples_to_vec!(rawbytes, samples, from_s24_4_be, 4);
            }
            SampleFormat::S32LE => {
                read_all_samples_to_vec!(rawbytes, samples, from_s32_le, 4);
            }
            SampleFormat::S32BE => {
                read_all_samples_to_vec!(rawbytes, samples, from_s32_be, 4);
            }
            SampleFormat::F32LE => {
                read_all_samples_to_vec!(rawbytes, samples, from_f32_le, 4);
            }
            SampleFormat::F32BE => {
                read_all_samples_to_vec!(rawbytes, samples, from_f32_be, 4);
            }
            SampleFormat::F64LE => {
                read_all_samples_to_vec!(rawbytes, samples, from_f64_le, 8);
            }
            SampleFormat::F64BE => {
                read_all_samples_to_vec!(rawbytes, samples, from_f64_be, 8);
            }
        }
        Ok(samples.len() - start_len)
    }
}

impl SampleReader<f64> for f64 {}
impl SampleReader<f32> for f32 {}

/// Clamp a float value to the range supported by an integer type
fn clamp_int<T: Float, U: Bounded + ToPrimitive>(value: T) -> (T, bool) {
    if value > T::from(U::max_value()).unwrap() {
        return (T::from(U::max_value()).unwrap(), true);
    } else if value < T::from(U::min_value()).unwrap() {
        return (T::from(U::min_value()).unwrap(), true);
    }
    (value, false)
}

/// Clamp a float value to the -1.0 .. +1.0
fn clamp_float<T: Float>(value: T) -> (T, bool) {
    if value >= T::one() {
        return (T::one(), true);
    } else if value < -T::one() {
        return (-T::one(), true);
    }
    (value, false)
}

impl Sample<f64> for f64 {
    const MAX_I32: f64 = 2147483648.0;
    const MAX_I24: f64 = 8388608.0;
    const MAX_I16: f64 = 32768.0;

    fn to_s16_le(&self) -> ([u8; 2], bool) {
        let val = self * f64::MAX_I16;
        let (val, clipped) = clamp_int::<f64, i16>(val);
        ((val as i16).to_le_bytes(), clipped)
    }

    fn to_s16_be(&self) -> ([u8; 2], bool) {
        let val = self * f64::MAX_I16;
        let (val, clipped) = clamp_int::<f64, i16>(val);
        ((val as i16).to_be_bytes(), clipped)
    }

    fn to_s32_le(&self) -> ([u8; 4], bool) {
        let val = self * f64::MAX_I32;
        let (val, clipped) = clamp_int::<f64, i32>(val);
        ((val as i32).to_le_bytes(), clipped)
    }

    fn to_s32_be(&self) -> ([u8; 4], bool) {
        let val = self * f64::MAX_I32;
        let (val, clipped) = clamp_int::<f64, i32>(val);
        ((val as i32).to_be_bytes(), clipped)
    }

    fn to_s24_3_le(&self) -> ([u8; 3], bool) {
        let val = self * f64::MAX_I32;
        let (val, clipped) = clamp_int::<f64, i32>(val);
        let bytes = (val as i32).to_le_bytes();
        ([bytes[1], bytes[2], bytes[3]], clipped)
    }

    fn to_s24_3_be(&self) -> ([u8; 3], bool) {
        let val = self * f64::MAX_I32;
        let (val, clipped) = clamp_int::<f64, i32>(val);
        let bytes = (val as i32).to_be_bytes();
        ([bytes[0], bytes[1], bytes[2]], clipped)
    }

    fn to_s24_4_le(&self) -> ([u8; 4], bool) {
        let val = self * f64::MAX_I32;
        let (val, clipped) = clamp_int::<f64, i32>(val);
        let bytes = (val as i32).to_le_bytes();
        ([bytes[1], bytes[2], bytes[3], 0], clipped)
    }

    fn to_s24_4_be(&self) -> ([u8; 4], bool) {
        let val = self * f64::MAX_I32;
        let (val, clipped) = clamp_int::<f64, i32>(val);
        let bytes = (val as i32).to_be_bytes();
        ([0, bytes[0], bytes[1], bytes[2]], clipped)
    }

    fn to_f64_le(&self) -> ([u8; 8], bool) {
        let val = *self;
        let (val, clipped) = clamp_float(val);
        (val.to_le_bytes(), clipped)
    }

    fn to_f64_be(&self) -> ([u8; 8], bool) {
        let val = *self;
        let (val, clipped) = clamp_float(val);
        (val.to_be_bytes(), clipped)
    }

    fn to_f32_le(&self) -> ([u8; 4], bool) {
        let val = *self as f32;
        let (val, clipped) = clamp_float(val);
        (val.to_le_bytes(), clipped)
    }

    fn to_f32_be(&self) -> ([u8; 4], bool) {
        let val = *self as f32;
        let (val, clipped) = clamp_float(val);
        (val.to_be_bytes(), clipped)
    }

    fn from_s32_le(bytes: [u8; 4]) -> Self {
        let intvalue = i32::from_le_bytes(bytes);
        f64::from(intvalue) / f64::MAX_I32
    }

    fn from_s32_be(bytes: [u8; 4]) -> Self {
        let intvalue = i32::from_be_bytes(bytes);
        f64::from(intvalue) / f64::MAX_I32
    }

    fn from_s16_le(bytes: [u8; 2]) -> Self {
        let intvalue = i16::from_le_bytes(bytes);
        f64::from(intvalue) / f64::MAX_I16
    }

    fn from_s16_be(bytes: [u8; 2]) -> Self {
        let intvalue = i16::from_be_bytes(bytes);
        f64::from(intvalue) / f64::MAX_I16
    }

    fn from_s24_3_le(bytes: [u8; 3]) -> Self {
        let padded = [0, bytes[0], bytes[1], bytes[2]];
        let intvalue = i32::from_le_bytes(padded);
        f64::from(intvalue) / f64::MAX_I32
    }

    fn from_s24_3_be(bytes: [u8; 3]) -> Self {
        let padded = [bytes[0], bytes[1], bytes[2], 0];
        let intvalue = i32::from_be_bytes(padded);
        f64::from(intvalue) / f64::MAX_I32
    }

    fn from_s24_4_le(bytes: [u8; 4]) -> Self {
        let padded = [0, bytes[0], bytes[1], bytes[2]];
        let intvalue = i32::from_le_bytes(padded);
        f64::from(intvalue) / f64::MAX_I32
    }

    fn from_s24_4_be(bytes: [u8; 4]) -> Self {
        let padded = [bytes[1], bytes[2], bytes[3], 0];
        let intvalue = i32::from_be_bytes(padded);
        f64::from(intvalue) / f64::MAX_I32
    }

    fn from_f32_le(bytes: [u8; 4]) -> Self {
        f64::from(f32::from_le_bytes(bytes))
    }

    fn from_f32_be(bytes: [u8; 4]) -> Self {
        f64::from(f32::from_be_bytes(bytes))
    }

    fn from_f64_le(bytes: [u8; 8]) -> Self {
        f64::from_le_bytes(bytes)
    }

    fn from_f64_be(bytes: [u8; 8]) -> Self {
        f64::from_be_bytes(bytes)
    }
}

impl Sample<f32> for f32 {
    const MAX_I32: f32 = 2147483648.0;
    const MAX_I24: f32 = 8388608.0;
    const MAX_I16: f32 = 32768.0;

    fn to_s16_le(&self) -> ([u8; 2], bool) {
        let val = self * f32::MAX_I16;
        let (val, clipped) = clamp_int::<f32, i16>(val);
        ((val as i16).to_le_bytes(), clipped)
    }

    fn to_s16_be(&self) -> ([u8; 2], bool) {
        let val = self * f32::MAX_I16;
        let (val, clipped) = clamp_int::<f32, i16>(val);
        ((val as i16).to_be_bytes(), clipped)
    }

    fn to_s32_le(&self) -> ([u8; 4], bool) {
        let val = self * f32::MAX_I32;
        let (val, clipped) = clamp_int::<f32, i32>(val);
        ((val as i32).to_le_bytes(), clipped)
    }

    fn to_s32_be(&self) -> ([u8; 4], bool) {
        let val = self * f32::MAX_I32;
        let (val, clipped) = clamp_int::<f32, i32>(val);
        ((val as i32).to_be_bytes(), clipped)
    }

    fn to_s24_3_le(&self) -> ([u8; 3], bool) {
        let val = self * f32::MAX_I32;
        let (val, clipped) = clamp_int::<f32, i32>(val);
        let bytes = (val as i32).to_le_bytes();
        ([bytes[1], bytes[2], bytes[3]], clipped)
    }

    fn to_s24_3_be(&self) -> ([u8; 3], bool) {
        let val = self * f32::MAX_I32;
        let (val, clipped) = clamp_int::<f32, i32>(val);
        let bytes = (val as i32).to_be_bytes();
        ([bytes[0], bytes[1], bytes[2]], clipped)
    }

    fn to_s24_4_le(&self) -> ([u8; 4], bool) {
        let val = self * f32::MAX_I32;
        let (val, clipped) = clamp_int::<f32, i32>(val);
        let bytes = (val as i32).to_le_bytes();
        ([bytes[1], bytes[2], bytes[3], 0], clipped)
    }

    fn to_s24_4_be(&self) -> ([u8; 4], bool) {
        let val = self * f32::MAX_I32;
        let (val, clipped) = clamp_int::<f32, i32>(val);
        let bytes = (val as i32).to_be_bytes();
        ([0, bytes[0], bytes[1], bytes[2]], clipped)
    }

    fn to_f64_le(&self) -> ([u8; 8], bool) {
        let val = f64::from(*self);
        let (val, clipped) = clamp_float(val);
        (val.to_le_bytes(), clipped)
    }

    fn to_f64_be(&self) -> ([u8; 8], bool) {
        let val = f64::from(*self);
        let (val, clipped) = clamp_float(val);
        (val.to_be_bytes(), clipped)
    }

    fn to_f32_le(&self) -> ([u8; 4], bool) {
        let (val, clipped) = clamp_float(*self);
        (val.to_le_bytes(), clipped)
    }

    fn to_f32_be(&self) -> ([u8; 4], bool) {
        let (val, clipped) = clamp_float(*self);
        (val.to_be_bytes(), clipped)
    }

    fn from_s32_le(bytes: [u8; 4]) -> Self {
        let intvalue = i32::from_le_bytes(bytes);
        intvalue as f32 / f32::MAX_I32
    }

    fn from_s32_be(bytes: [u8; 4]) -> Self {
        let intvalue = i32::from_be_bytes(bytes);
        intvalue as f32 / f32::MAX_I32
    }

    fn from_s16_le(bytes: [u8; 2]) -> Self {
        let intvalue = i16::from_le_bytes(bytes);
        f32::from(intvalue) / f32::MAX_I16
    }

    fn from_s16_be(bytes: [u8; 2]) -> Self {
        let intvalue = i16::from_be_bytes(bytes);
        f32::from(intvalue) / f32::MAX_I16
    }

    fn from_s24_3_le(bytes: [u8; 3]) -> Self {
        let padded = [0, bytes[0], bytes[1], bytes[2]];
        let intvalue = i32::from_le_bytes(padded);
        intvalue as f32 / f32::MAX_I32
    }

    fn from_s24_3_be(bytes: [u8; 3]) -> Self {
        let padded = [bytes[0], bytes[1], bytes[2], 0];
        let intvalue = i32::from_be_bytes(padded);
        intvalue as f32 / f32::MAX_I32
    }

    fn from_s24_4_le(bytes: [u8; 4]) -> Self {
        let padded = [0, bytes[0], bytes[1], bytes[2]];
        let intvalue = i32::from_le_bytes(padded);
        intvalue as f32 / f32::MAX_I32
    }

    fn from_s24_4_be(bytes: [u8; 4]) -> Self {
        let padded = [bytes[1], bytes[2], bytes[3], 0];
        let intvalue = i32::from_be_bytes(padded);
        intvalue as f32 / f32::MAX_I32
    }

    fn from_f32_le(bytes: [u8; 4]) -> Self {
        f32::from_le_bytes(bytes)
    }

    fn from_f32_be(bytes: [u8; 4]) -> Self {
        f32::from_be_bytes(bytes)
    }

    fn from_f64_le(bytes: [u8; 8]) -> Self {
        f64::from_le_bytes(bytes) as f32
    }

    fn from_f64_be(bytes: [u8; 8]) -> Self {
        f64::from_be_bytes(bytes) as f32
    }
}

#[cfg(test)]
mod tests {
    use crate::Sample;
    use crate::SampleFormat;
    use crate::SampleReader;
    use crate::SampleWriter;

    // -------------------
    //  single values f64
    // -------------------

    #[test]
    fn check_f64_to_s32le() {
        let val: f64 = 0.256789;
        assert_eq!(val.to_s32_le(), ([66, 118, 222, 32], false));
        let val: f64 = -0.256789;
        assert_eq!(val.to_s32_le(), ([190, 137, 33, 223], false));
        let val: f64 = 1.1;
        assert_eq!(val.to_s32_le(), ([255, 255, 255, 127], true));
        let val: f64 = -1.1;
        assert_eq!(val.to_s32_le(), ([0, 0, 0, 128], true));
    }

    #[test]
    fn check_f64_from_s32le() {
        let data = [0, 0, 64, 32];
        assert_eq!(f64::from_s32_le(data), 0.251953125);
        let data = [0, 0, 64, 223];
        assert_eq!(f64::from_s32_le(data), -0.255859375);
        let data = [0, 0, 0, 128];
        assert_eq!(f64::from_s32_le(data), -1.0);
    }

    #[test]
    fn check_f64_to_s32be() {
        let val: f64 = 0.256789;
        assert_eq!(val.to_s32_be(), ([32, 222, 118, 66], false));
        let val: f64 = -0.256789;
        assert_eq!(val.to_s32_be(), ([223, 33, 137, 190], false));
        let val: f64 = 1.1;
        assert_eq!(val.to_s32_be(), ([127, 255, 255, 255], true));
        let val: f64 = -1.1;
        assert_eq!(val.to_s32_be(), ([128, 0, 0, 0], true));
    }

    #[test]
    fn check_f64_from_s32be() {
        let data = [32, 64, 0, 0];
        assert_eq!(f64::from_s32_be(data), 0.251953125);
        let data = [223, 64, 0, 0];
        assert_eq!(f64::from_s32_be(data), -0.255859375);
        let data = [128, 0, 0, 0];
        assert_eq!(f64::from_s32_be(data), -1.0);
    }

    #[test]
    fn check_f64_to_s243le() {
        let val: f64 = 0.256789;
        assert_eq!(val.to_s24_3_le(), ([118, 222, 32], false));
        let val: f64 = -0.256789;
        assert_eq!(val.to_s24_3_le(), ([137, 33, 223], false));
        let val: f64 = 1.1;
        assert_eq!(val.to_s24_3_le(), ([255, 255, 127], true));
        let val: f64 = -1.1;
        assert_eq!(val.to_s24_3_le(), ([0, 0, 128], true));
    }

    #[test]
    fn check_f64_to_s243be() {
        let val: f64 = 0.256789;
        assert_eq!(val.to_s24_3_be(), ([32, 222, 118], false));
        let val: f64 = -0.256789;
        assert_eq!(val.to_s24_3_be(), ([223, 33, 137], false));
        let val: f64 = 1.1;
        assert_eq!(val.to_s24_3_be(), ([127, 255, 255], true));
        let val: f64 = -1.1;
        assert_eq!(val.to_s24_3_be(), ([128, 0, 0], true));
    }

    #[test]
    fn check_f64_from_s243le() {
        let data = [0, 64, 32];
        assert_eq!(f64::from_s24_3_le(data), 0.251953125);
        let data = [0, 64, 223];
        assert_eq!(f64::from_s24_3_le(data), -0.255859375);
        let data = [0, 0, 128];
        assert_eq!(f64::from_s24_3_le(data), -1.0);
    }

    #[test]
    fn check_f64_from_s243be() {
        let data = [32, 64, 0];
        assert_eq!(f64::from_s24_3_be(data), 0.251953125);
        let data = [223, 64, 0];
        assert_eq!(f64::from_s24_3_be(data), -0.255859375);
        let data = [128, 0, 0];
        assert_eq!(f64::from_s24_3_be(data), -1.0);
    }

    #[test]
    fn check_f64_from_s244le() {
        let data = [0, 64, 32, 0];
        assert_eq!(f64::from_s24_4_le(data), 0.251953125);
        let data = [0, 64, 223, 0];
        assert_eq!(f64::from_s24_4_le(data), -0.255859375);
        let data = [0, 0, 128, 0];
        assert_eq!(f64::from_s24_4_le(data), -1.0);
    }

    #[test]
    fn check_f64_from_s244be() {
        let data = [0, 32, 64, 0];
        assert_eq!(f64::from_s24_4_be(data), 0.251953125);
        let data = [0, 223, 64, 0];
        assert_eq!(f64::from_s24_4_be(data), -0.255859375);
        let data = [0, 128, 0, 0];
        assert_eq!(f64::from_s24_4_be(data), -1.0);
    }

    #[test]
    fn check_f64_to_s244le() {
        let val: f64 = 0.256789;
        assert_eq!(val.to_s24_4_le(), ([118, 222, 32, 0], false));
        let val: f64 = -0.256789;
        assert_eq!(val.to_s24_4_le(), ([137, 33, 223, 0], false));
        let val: f64 = 1.1;
        assert_eq!(val.to_s24_4_le(), ([255, 255, 127, 0], true));
        let val: f64 = -1.1;
        assert_eq!(val.to_s24_4_le(), ([0, 0, 128, 0], true));
    }

    #[test]
    fn check_f64_to_s244be() {
        let val: f64 = 0.256789;
        assert_eq!(val.to_s24_4_be(), ([0, 32, 222, 118], false));
        let val: f64 = -0.256789;
        assert_eq!(val.to_s24_4_be(), ([0, 223, 33, 137], false));
        let val: f64 = 1.1;
        assert_eq!(val.to_s24_4_be(), ([0, 127, 255, 255], true));
        let val: f64 = -1.1;
        assert_eq!(val.to_s24_4_be(), ([0, 128, 0, 0], true));
    }

    #[test]
    fn check_f64_to_s16le() {
        let val: f64 = 0.256789;
        assert_eq!(val.to_s16_le(), ([222, 32], false));
        let val: f64 = -0.256789;
        assert_eq!(val.to_s16_le(), ([34, 223], false));
    }

    #[test]
    fn check_f64_to_s16be() {
        let val: f64 = 0.256789;
        assert_eq!(val.to_s16_be(), ([32, 222], false));
        let val: f64 = -0.256789;
        assert_eq!(val.to_s16_be(), ([223, 34], false));
    }

    #[test]
    fn check_f64_to_f32le() {
        let val: f64 = 0.256789;
        let exp = (0.256789 as f32).to_le_bytes();
        assert_eq!(val.to_f32_le(), (exp, false));
        let val: f64 = -0.256789;
        let exp = (-0.256789 as f32).to_le_bytes();
        assert_eq!(val.to_f32_le(), (exp, false));
        let val: f64 = 1.1;
        let exp = (1.0 as f32).to_le_bytes();
        assert_eq!(val.to_f32_le(), (exp, true));
        let val: f64 = -1.1;
        let exp = (-1.0 as f32).to_le_bytes();
        assert_eq!(val.to_f32_le(), (exp, true));
    }

    #[test]
    fn check_f64_to_f32be() {
        let val: f64 = 0.256789;
        let exp = (0.256789 as f32).to_be_bytes();
        assert_eq!(val.to_f32_be(), (exp, false));
        let val: f64 = -0.256789;
        let exp = (-0.256789 as f32).to_be_bytes();
        assert_eq!(val.to_f32_be(), (exp, false));
        let val: f64 = 1.1;
        let exp = (1.0 as f32).to_be_bytes();
        assert_eq!(val.to_f32_be(), (exp, true));
        let val: f64 = -1.1;
        let exp = (-1.0 as f32).to_be_bytes();
        assert_eq!(val.to_f32_be(), (exp, true));
    }

    #[test]
    fn check_f64_to_f64le() {
        let val: f64 = 0.256789;
        let exp = (0.256789 as f64).to_le_bytes();
        assert_eq!(val.to_f64_le(), (exp, false));
        let val: f64 = -0.256789;
        let exp = (-0.256789 as f64).to_le_bytes();
        assert_eq!(val.to_f64_le(), (exp, false));
        let val: f64 = 1.1;
        let exp = (1.0 as f64).to_le_bytes();
        assert_eq!(val.to_f64_le(), (exp, true));
        let val: f64 = -1.1;
        let exp = (-1.0 as f64).to_le_bytes();
        assert_eq!(val.to_f64_le(), (exp, true));
    }

    #[test]
    fn check_f64_to_f64be() {
        let val: f64 = 0.256789;
        let exp = (0.256789 as f64).to_be_bytes();
        assert_eq!(val.to_f64_be(), (exp, false));
        let val: f64 = -0.256789;
        let exp = (-0.256789 as f64).to_be_bytes();
        assert_eq!(val.to_f64_be(), (exp, false));
        let val: f64 = 1.1;
        let exp = (1.0 as f64).to_be_bytes();
        assert_eq!(val.to_f64_be(), (exp, true));
        let val: f64 = -1.1;
        let exp = (-1.0 as f64).to_be_bytes();
        assert_eq!(val.to_f64_be(), (exp, true));
    }

    // -------------------
    //  single values f32
    // -------------------
    #[test]
    fn check_f32_to_s32le() {
        let val: f32 = 0.256789;
        assert_eq!(val.to_s32_le(), ([64, 118, 222, 32], false));
        let val: f32 = -0.256789;
        assert_eq!(val.to_s32_le(), ([192, 137, 33, 223], false));
        let val: f32 = 1.1;
        assert_eq!(val.to_s32_le(), ([255, 255, 255, 127], true));
        let val: f32 = -1.1;
        assert_eq!(val.to_s32_le(), ([0, 0, 0, 128], true));
    }

    #[test]
    fn check_f32_from_s32le() {
        let data = [0, 0, 64, 32];
        assert_eq!(f32::from_s32_le(data), 0.251953125);
        let data = [0, 0, 64, 223];
        assert_eq!(f32::from_s32_le(data), -0.255859375);
        let data = [0, 0, 0, 128];
        assert_eq!(f32::from_s32_le(data), -1.0);
    }

    #[test]
    fn check_f32_to_s32be() {
        let val: f32 = 0.256789;
        assert_eq!(val.to_s32_be(), ([32, 222, 118, 64], false));
        let val: f32 = -0.256789;
        assert_eq!(val.to_s32_be(), ([223, 33, 137, 192], false));
        let val: f32 = 1.1;
        assert_eq!(val.to_s32_be(), ([127, 255, 255, 255], true));
        let val: f32 = -1.1;
        assert_eq!(val.to_s32_be(), ([128, 0, 0, 0], true));
    }

    #[test]
    fn check_f32_from_s32be() {
        let data = [32, 64, 0, 0];
        assert_eq!(f32::from_s32_be(data), 0.251953125);
        let data = [223, 64, 0, 0];
        assert_eq!(f32::from_s32_be(data), -0.255859375);
        let data = [128, 0, 0, 0];
        assert_eq!(f32::from_s32_be(data), -1.0);
    }

    #[test]
    fn check_f32_to_s243le() {
        let val: f32 = 0.256789;
        assert_eq!(val.to_s24_3_le(), ([118, 222, 32], false));
        let val: f32 = -0.256789;
        assert_eq!(val.to_s24_3_le(), ([137, 33, 223], false));
        let val: f32 = 1.1;
        assert_eq!(val.to_s24_3_le(), ([255, 255, 127], true));
        let val: f32 = -1.1;
        assert_eq!(val.to_s24_3_le(), ([0, 0, 128], true));
    }

    #[test]
    fn check_f32_to_s243be() {
        let val: f32 = 0.256789;
        assert_eq!(val.to_s24_3_be(), ([32, 222, 118], false));
        let val: f32 = -0.256789;
        assert_eq!(val.to_s24_3_be(), ([223, 33, 137], false));
        let val: f32 = 1.1;
        assert_eq!(val.to_s24_3_be(), ([127, 255, 255], true));
        let val: f32 = -1.1;
        assert_eq!(val.to_s24_3_be(), ([128, 0, 0], true));
    }

    #[test]
    fn check_f32_from_s243le() {
        let data = [0, 64, 32];
        assert_eq!(f32::from_s24_3_le(data), 0.251953125);
        let data = [0, 64, 223];
        assert_eq!(f32::from_s24_3_le(data), -0.255859375);
        let data = [0, 0, 128];
        assert_eq!(f32::from_s24_3_le(data), -1.0);
    }

    #[test]
    fn check_f32_from_s243be() {
        let data = [32, 64, 0];
        assert_eq!(f32::from_s24_3_be(data), 0.251953125);
        let data = [223, 64, 0];
        assert_eq!(f32::from_s24_3_be(data), -0.255859375);
        let data = [128, 0, 0];
        assert_eq!(f32::from_s24_3_be(data), -1.0);
    }

    #[test]
    fn check_f32_from_s244le() {
        let data = [0, 64, 32, 0];
        assert_eq!(f32::from_s24_4_le(data), 0.251953125);
        let data = [0, 64, 223, 0];
        assert_eq!(f32::from_s24_4_le(data), -0.255859375);
        let data = [0, 0, 128, 0];
        assert_eq!(f32::from_s24_4_le(data), -1.0);
    }

    #[test]
    fn check_f32_from_s244be() {
        let data = [0, 32, 64, 0];
        assert_eq!(f32::from_s24_4_be(data), 0.251953125);
        let data = [0, 223, 64, 0];
        assert_eq!(f32::from_s24_4_be(data), -0.255859375);
        let data = [0, 128, 0, 0];
        assert_eq!(f32::from_s24_4_be(data), -1.0);
    }

    #[test]
    fn check_f32_to_s244le() {
        let val: f32 = 0.256789;
        assert_eq!(val.to_s24_4_le(), ([118, 222, 32, 0], false));
        let val: f32 = -0.256789;
        assert_eq!(val.to_s24_4_le(), ([137, 33, 223, 0], false));
        let val: f32 = 1.1;
        assert_eq!(val.to_s24_4_le(), ([255, 255, 127, 0], true));
        let val: f32 = -1.1;
        assert_eq!(val.to_s24_4_le(), ([0, 0, 128, 0], true));
    }

    #[test]
    fn check_f32_to_s244be() {
        let val: f32 = 0.256789;
        assert_eq!(val.to_s24_4_be(), ([0, 32, 222, 118], false));
        let val: f32 = -0.256789;
        assert_eq!(val.to_s24_4_be(), ([0, 223, 33, 137], false));
        let val: f32 = 1.1;
        assert_eq!(val.to_s24_4_be(), ([0, 127, 255, 255], true));
        let val: f32 = -1.1;
        assert_eq!(val.to_s24_4_be(), ([0, 128, 0, 0], true));
    }

    #[test]
    fn check_f32_to_s16le() {
        let val: f32 = 0.256789;
        assert_eq!(val.to_s16_le(), ([222, 32], false));
        let val: f32 = -0.256789;
        assert_eq!(val.to_s16_le(), ([34, 223], false));
    }

    #[test]
    fn check_f32_to_s16be() {
        let val: f32 = 0.256789;
        assert_eq!(val.to_s16_be(), ([32, 222], false));
        let val: f32 = -0.256789;
        assert_eq!(val.to_s16_be(), ([223, 34], false));
    }

    #[test]
    fn check_f32_to_f32le() {
        let val: f32 = 0.256789;
        let exp = (0.256789 as f32).to_le_bytes();
        assert_eq!(val.to_f32_le(), (exp, false));
        let val: f32 = -0.256789;
        let exp = (-0.256789 as f32).to_le_bytes();
        assert_eq!(val.to_f32_le(), (exp, false));
        let val: f32 = 1.1;
        let exp = (1.0 as f32).to_le_bytes();
        assert_eq!(val.to_f32_le(), (exp, true));
        let val: f32 = -1.1;
        let exp = (-1.0 as f32).to_le_bytes();
        assert_eq!(val.to_f32_le(), (exp, true));
    }

    #[test]
    fn check_f32_to_f32be() {
        let val: f32 = 0.256789;
        let exp = (0.256789 as f32).to_be_bytes();
        assert_eq!(val.to_f32_be(), (exp, false));
        let val: f32 = -0.256789;
        let exp = (-0.256789 as f32).to_be_bytes();
        assert_eq!(val.to_f32_be(), (exp, false));
        let val: f32 = 1.1;
        let exp = (1.0 as f32).to_be_bytes();
        assert_eq!(val.to_f32_be(), (exp, true));
        let val: f32 = -1.1;
        let exp = (-1.0 as f32).to_be_bytes();
        assert_eq!(val.to_f32_be(), (exp, true));
    }

    #[test]
    fn check_f32_to_f64le() {
        let val: f32 = 0.256789;
        let exp = ((0.256789 as f32) as f64).to_le_bytes();
        assert_eq!(val.to_f64_le(), (exp, false));
        let val: f32 = -0.256789;
        let exp = ((-0.256789 as f32) as f64).to_le_bytes();
        assert_eq!(val.to_f64_le(), (exp, false));
        let val: f32 = 1.1;
        let exp = (1.0 as f64).to_le_bytes();
        assert_eq!(val.to_f64_le(), (exp, true));
        let val: f32 = -1.1;
        let exp = (-1.0 as f64).to_le_bytes();
        assert_eq!(val.to_f64_le(), (exp, true));
    }

    #[test]
    fn check_f32_to_f64be() {
        let val: f32 = 0.256789;
        let exp = ((0.256789 as f32) as f64).to_be_bytes();
        assert_eq!(val.to_f64_be(), (exp, false));
        let val: f32 = -0.256789;
        let exp = ((-0.256789 as f32) as f64).to_be_bytes();
        assert_eq!(val.to_f64_be(), (exp, false));
        let val: f32 = 1.1;
        let exp = (1.0 as f64).to_be_bytes();
        assert_eq!(val.to_f64_be(), (exp, true));
        let val: f32 = -1.1;
        let exp = (-1.0 as f64).to_be_bytes();
        assert_eq!(val.to_f64_be(), (exp, true));
    }

    // -----------------
    //  read/write many
    // -----------------

    #[test]
    fn write_read_to_slice_s16le() {
        // write data, then read it back into a slice of the same length.
        let values = vec![-0.5, -0.25, -0.125, 0.0, 0.125, 0.25, 0.5];
        let mut data: Vec<u8> = Vec::new();
        f64::write_samples(&values, &mut data, &SampleFormat::S16LE).unwrap();
        let mut values2 = vec![0.0; 7];
        let mut slice: &[u8] = &data;
        f64::read_samples(&mut slice, &mut values2, &SampleFormat::S16LE).unwrap();
        assert_eq!(values, values2);
    }

    #[test]
    fn write_read_to_slice_s16be() {
        // write data, then read it back into a slice of the same length.
        let values = vec![-0.5, -0.25, -0.125, 0.0, 0.125, 0.25, 0.5];
        let mut data: Vec<u8> = Vec::new();
        f64::write_samples(&values, &mut data, &SampleFormat::S16BE).unwrap();
        let mut values2 = vec![0.0; 7];
        let mut slice: &[u8] = &data;
        f64::read_samples(&mut slice, &mut values2, &SampleFormat::S16BE).unwrap();
        assert_eq!(values, values2);
    }

    #[test]
    fn write_read_all_s32le() {
        // write data, then read all of it back into a dynamically allocated vec.
        let values = vec![-0.5, -0.25, -0.125, 0.0, 0.125, 0.25, 0.5];
        let mut data: Vec<u8> = Vec::new();
        f64::write_samples(&values, &mut data, &SampleFormat::S32LE).unwrap();
        let mut values2 = Vec::new();
        let mut slice: &[u8] = &data;
        f64::read_all_samples(&mut slice, &mut values2, &SampleFormat::S32LE).unwrap();
        assert_eq!(values, values2);
    }

    #[test]
    fn write_read_all_s32be() {
        // write data, then read all of it back into a dynamically allocated vec.
        let values = vec![-0.5, -0.25, -0.125, 0.0, 0.125, 0.25, 0.5];
        let mut data: Vec<u8> = Vec::new();
        f64::write_samples(&values, &mut data, &SampleFormat::S32BE).unwrap();
        let mut values2 = Vec::new();
        let mut slice: &[u8] = &data;
        f64::read_all_samples(&mut slice, &mut values2, &SampleFormat::S32BE).unwrap();
        assert_eq!(values, values2);
    }

    #[test]
    fn read_to_shorter_slice_s16le() {
        // reading into a shorter slice should skip reading the last samples.
        let values = vec![-0.5, -0.25, -0.125, 0.0, 0.125, 0.25, 0.5];
        let mut data: Vec<u8> = Vec::new();
        f64::write_samples(&values, &mut data, &SampleFormat::S16LE).unwrap();
        let mut values2 = vec![0.0; 6];
        let mut slice: &[u8] = &data;
        f64::read_samples(&mut slice, &mut values2, &SampleFormat::S16LE).unwrap();
        assert_eq!(values[0..6], values2);
    }

    #[test]
    fn read_to_longer_slice_s16le() {
        // reading into a longer slice should not change the unused part of the slice.
        let values = vec![-0.5, -0.25, -0.125, 0.0, 0.125, 0.25, 0.5];
        let mut data: Vec<u8> = Vec::new();
        f64::write_samples(&values, &mut data, &SampleFormat::S16LE).unwrap();
        let mut values2 = vec![0.75; 9];
        let mut slice: &[u8] = &data;
        f64::read_samples(&mut slice, &mut values2, &SampleFormat::S16LE).unwrap();
        let expected = vec![-0.5, -0.25, -0.125, 0.0, 0.125, 0.25, 0.5, 0.75, 0.75];
        assert_eq!(expected, values2);
    }
}
