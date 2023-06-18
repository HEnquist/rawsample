//! # Converting wrappers
//! This module provides wrappers for slices of bytes.
//! The wrapper enables reading and writing samples from/to the byte slice with
//! on-the-fly format conversion.
//!
//! The wrappers implement the traits [wrapper::Converter] and [wrapper::ConverterMut],
//! that provide simple methods for accessing the audio samples of a buffer.
//!
//! ## Abstracting the data layout
//!
//! ### Channels and frames
//! When audio data has more than one channel, it is made up of a series of _frames_.
//! A frame consists of the samples for all channels, belonging to one time point.
//! For normal stereo, a frame consists of one sample for the left channel
//! and one for the right, usually in that order.
//!
//! ### Interleaved and sequential layout
//! When the audio data is stored in a file or in memory,
//! the data can be arranged in two main ways.
//! - Keeping all samples for each channel together,
//!   and storing each channel after the previous.
//!   This is normally called _sequential_, _non-interleaved_ or _planar_.
//!   The sample order of a stereo file with 3 frames becomes:
//!   `L1, L2, L3, R1, R2, R3`
//! - Keeping all samples for each frame together,
//!   and storing each frame after the previous.
//!   This is normally called _interleaved_, and this is how the data in a .wav file is ordered.
//!   The sample order of a stereo file with 3 frames becomes:
//!   `L1, R1, L2, R2, L3, R3`
//!
//! ### Wrappers
//! There are two wrappers availabe for each sample format,
//! one for interleaved and one for sequential data.
//! By using the appropriate wrapper, the sample values can be
//! easily accessed via the trait methods, which means an application
//! can handle both layouts without any extra code.
//!
//! ### Example
//! Wrap a Vec of bytes as an interleaved buffer of 16-bit integer samples
//! and print all the values.
//! ```
//! use rawsample::wrapper::{Converter, InterleavedS16LE};
//!
//! // make a vector with some fake data.
//! // 2 channels * 3 frames * 2 bytes per sample => 12 bytes
//! let data: Vec<u8> = vec![0, 0, 0, 128, 0, 64, 0, 192, 0, 32, 0, 224];
//!
//! // wrap the data
//! let buffer: InterleavedS16LE<&[u8], f32> = InterleavedS16LE::new(&data, 2, 3).unwrap();
//!
//! // Loop over all samples and print their values
//! for channel in 0..2 {
//!     for frame in 0..3 {
//!         let value = buffer.read(channel, frame).unwrap();
//!         println!(
//!             "Channel: {}, frame: {}, value: {}",
//!             channel, frame, value
//!         );
//!     }
//! }
//! ```

use std::convert::TryInto;
use std::error;
use std::fmt;

use crate::Sample;

/// Error returned when the wrapped data structure has the wrong dimensions,
/// typically that it is too short.
#[derive(Debug)]
pub struct BufferSizeError {
    desc: String,
}

impl fmt::Display for BufferSizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.desc)
    }
}

impl error::Error for BufferSizeError {
    fn description(&self) -> &str {
        &self.desc
    }
}

impl BufferSizeError {
    pub fn new(desc: &str) -> Self {
        BufferSizeError {
            desc: desc.to_owned(),
        }
    }
}

macro_rules! implement_size_getters {
    () => {
        fn channels(&self) -> usize {
            self.channels
        }

        fn frames(&self) -> usize {
            self.frames
        }
    };
}

macro_rules! check_slice_length {
    ($channels:expr , $frames:expr, $length:expr, $elements_per_sample:expr) => {
        if $length < $frames * $channels * $elements_per_sample {
            return Err(BufferSizeError {
                desc: format!(
                    "Slice is too short, {} < {}",
                    $length,
                    $frames * $channels * $elements_per_sample
                ),
            });
        }
    };
}

// -------------------- The main buffer trait --------------------

/// A trait for providing immutable access to samples in a buffer.
/// Samples are converted from the raw format on the fly.
pub trait Converter<'a, T: 'a> {
    /// Read and convert the sample at
    /// a given combination of frame and channel.
    ///
    /// # Safety
    ///
    /// This method performs no bounds checking.
    /// Calling it with an out-of-bound value for frame or channel
    /// results in undefined behavior,
    /// for example returning an invalid value or panicking.
    unsafe fn read_unchecked(&self, channel: usize, frame: usize) -> T;

    /// Read and convert the sample at
    /// a given combination of frame and channel.
    /// Returns `None` if the frame or channel is
    /// out of bounds of the `Converter`.
    fn read(&self, channel: usize, frame: usize) -> Option<T> {
        if channel >= self.channels() || frame >= self.frames() {
            return None;
        }
        Some(unsafe { self.read_unchecked(channel, frame) })
    }

    /// Get the number of channels stored in this `Converter`.
    fn channels(&self) -> usize;

    /// Get the number of frames stored in this `Converter`.
    fn frames(&self) -> usize;

    /// Convert and write values from a channel of the `Converter` to a slice.
    /// The `start` argument is the offset into the `Converter` channel
    /// where the first value will be read from.
    /// If the slice is longer than the available number of values in the `Converter` channel,
    /// then only the available number of samples will be written.
    ///
    /// Returns the number of values written.
    /// If an invalid channel number is given,
    /// or if `start` is larger than the length of the channel,
    /// no samples will be written and zero is returned.
    fn write_from_channel_to_slice(&self, channel: usize, start: usize, slice: &mut [T]) -> usize {
        if channel >= self.channels() || start >= self.frames() {
            return 0;
        }
        let frames_to_write = if (self.frames() - start) < slice.len() {
            self.frames() - start
        } else {
            slice.len()
        };
        for (n, item) in slice.iter_mut().enumerate().take(frames_to_write) {
            unsafe { *item = self.read_unchecked(channel, start + n) };
        }
        frames_to_write
    }

    /// Convert and write values from a frame of the `Converter` to a slice.
    /// The `start` argument is the offset into the `Converter` frame
    /// where the first value will be read from.
    /// If the slice is longer than the available number of values in the `Converter` frame,
    /// then only the available number of samples will be written.
    ///
    /// Returns the number of values written.
    /// If an invalid frame number is given,
    /// or if `start` is larger than the length of the frame,
    /// no samples will be written and zero is returned.
    fn write_from_frame_to_slice(&self, frame: usize, start: usize, slice: &mut [T]) -> usize {
        if frame >= self.frames() || start >= self.channels() {
            return 0;
        }
        let channels_to_write = if (self.channels() - start) < slice.len() {
            self.channels() - start
        } else {
            slice.len()
        };
        for (n, item) in slice.iter_mut().enumerate().take(channels_to_write) {
            unsafe { *item = self.read_unchecked(start + n, frame) };
        }
        channels_to_write
    }
}

/// A trait for providing mutable access to samples in a buffer.
/// Samples are converted to the raw format on the fly.
pub trait ConverterMut<'a, T>: Converter<'a, T>
where
    T: Clone + 'a,
{
    /// Convert and write a sample to the
    /// given combination of frame and channel.
    ///
    /// # Safety
    ///
    /// This method performs no bounds checking.
    /// Calling it with an out-of-bound value for frame or channel
    /// results in undefined behavior,
    /// for example returning an invalid value or panicking.
    unsafe fn write_unchecked(&mut self, channel: usize, frame: usize, value: &T) -> bool;

    /// Convert and write a sample to the
    /// given combination of frame and channel.
    /// Returns `None` if the frame or channel is
    /// out of bounds of the `Converter`.
    fn write(&mut self, channel: usize, frame: usize, value: &T) -> Option<bool> {
        if channel >= self.channels() || frame >= self.frames() {
            return None;
        }
        Some(unsafe { self.write_unchecked(channel, frame, value) })
    }

    /// Write values from a slice into a channel of the `Converter`.
    /// The `start` argument is the offset into the `Converter` channel
    /// where the first value will be written.
    /// If the slice is longer than the available space in the `Converter` channel,
    /// then only the number of samples that fit will be read.
    ///
    /// Returns a tuple of two numbers.
    /// The first is the number of values written,
    /// and the second is the number of values that were clipped during conversion.
    /// If an invalid channel number is given,
    /// or if `start` is larger than the length of the channel,
    /// no samples will be read and (0, 0) is returned.
    fn write_from_slice_to_channel(
        &mut self,
        channel: usize,
        start: usize,
        slice: &[T],
    ) -> (usize, usize) {
        if channel >= self.channels() || start >= self.frames() {
            return (0, 0);
        }
        let frames_to_read = if (self.frames() - start) < slice.len() {
            self.frames() - start
        } else {
            slice.len()
        };
        let mut nbr_clipped = 0;
        for (n, item) in slice.iter().enumerate().take(frames_to_read) {
            unsafe { nbr_clipped += self.write_unchecked(channel, start + n, item) as usize };
        }
        (frames_to_read, nbr_clipped)
    }

    /// Write values from a slice into a frame of the `Converter`.
    /// The `start` argument is the offset into the `Converter` frame
    /// where the first value will be written.
    /// If the slice is longer than the available space in the `Converter` frame,
    /// then only the number of samples that fit will be read.
    ///
    /// Returns a tuple of two numbers.
    /// The first is the number of values written,
    /// and the second is the number of values that were clipped during conversion.
    /// If an invalid frame number is given,
    /// or if `start` is larger than the length of the frame,
    /// no samples will be read and (0, 0) is returned.
    fn write_from_slice_to_frame(
        &mut self,
        frame: usize,
        start: usize,
        slice: &[T],
    ) -> (usize, usize) {
        if frame >= self.frames() || start >= self.channels() {
            return (0, 0);
        }
        let channels_to_read = if (self.channels() - start) < slice.len() {
            self.channels() - start
        } else {
            slice.len()
        };
        let mut nbr_clipped = 0;
        for (n, item) in slice.iter().enumerate().take(channels_to_read) {
            unsafe { nbr_clipped += self.write_unchecked(start + n, frame, item) as usize };
        }
        (channels_to_read, nbr_clipped)
    }
}

macro_rules! create_structs {
    ($type:expr, $read_func:ident, $write_func:ident, $bytes:expr, $typename:ident) => {
        paste::item! {
            #[doc = "A wrapper for a slice of bytes containing interleaved samples in the `" $typename "` format."]
            pub struct [< Interleaved $typename >]<U, V> {
                _phantom: core::marker::PhantomData<V>,
                buf: U,
                frames: usize,
                channels: usize,
                bytes_per_sample: usize,
            }

            #[doc = "A wrapper for a slice of bytes containing sequential samples in the `" $typename "` format."]
            pub struct [< Sequential $typename >]<U, V> {
                _phantom: core::marker::PhantomData<V>,
                buf: U,
                frames: usize,
                channels: usize,
                bytes_per_sample: usize,
            }

            impl<U, V> [< Interleaved $typename >]<U, V> {
                fn calc_index(&self, channel: usize, frame: usize) -> usize {
                    self.bytes_per_sample * (frame * self.channels + channel)
                }
            }

            impl<U, V> [< Sequential $typename >]<U, V> {
                fn calc_index(&self, channel: usize, frame: usize) -> usize {
                    self.bytes_per_sample * (frame + channel * self.frames)
                }
            }
        }
    };
}

macro_rules! impl_traits {
    ($type:expr, $read_func:ident, $write_func:ident, $bytes:expr, $typename:ident, $order:ident) => {
        paste::item! {


            impl<'a, T> [< $order $typename >]<&'a [u8], T>
            where
                T: 'a,
            {
                #[doc = "Create a new wrapper for a slice of bytes,"]
                #[doc = "containing samples of type `" $typename "`,"]
                #[doc = "stored in _" $order:lower "_ order."]
                #[doc = "The slice length must be at least `" $bytes "*frames*channels`."]
                #[doc = "It is allowed to be longer than needed,"]
                #[doc = "but these extra values cannot"]
                #[doc = "be accessed via the `Converter` trait methods."]
                pub fn new(
                    buf: &'a [u8],
                    channels: usize,
                    frames: usize,
                ) -> Result<Self, BufferSizeError> {
                    check_slice_length!(channels, frames, buf.len(), $bytes);
                    Ok(Self {
                        _phantom: core::marker::PhantomData,
                        buf,
                        frames,
                        channels,
                        bytes_per_sample: $bytes,
                    })
                }
            }

            impl<'a, T> [< $order $typename >]<&'a mut [u8], T>
            where
                T: 'a,
            {
                #[doc = "Create a new wrapper for a mutable slice of bytes,"]
                #[doc = "containing samples of type `" $typename "`,"]
                #[doc = "stored in _" $order:lower "_ order."]
                #[doc = "The slice length must be at least `" $bytes " *frames*channels`."]
                #[doc = "It is allowed to be longer than needed,"]
                #[doc = "but these extra values cannot"]
                #[doc = "be accessed via the `Converter` trait methods."]
                pub fn new_mut(
                    buf: &'a mut [u8],
                    channels: usize,
                    frames: usize,
                ) -> Result<Self, BufferSizeError> {
                    check_slice_length!(channels, frames, buf.len(), $bytes);
                    Ok(Self {
                        _phantom: core::marker::PhantomData,
                        buf,
                        frames,
                        channels,
                        bytes_per_sample: $bytes,
                    })
                }
            }

            impl<'a, T> Converter<'a, T> for [< $order $typename >]<&'a [u8], T>
            where
                T: Sample<T> + 'a,
            {
                unsafe fn read_unchecked(&self, channel: usize, frame: usize) -> T {
                    let index = self.calc_index(channel, frame);
                    T::$read_func(
                        self.buf[index..index + self.bytes_per_sample]
                            .try_into()
                            .unwrap(),
                    )
                }

                implement_size_getters!();
            }

            impl<'a, T> Converter<'a, T> for [< $order $typename >]<&'a mut [u8], T>
            where
                T: Sample<T> + Clone + 'a,
            {
                unsafe fn read_unchecked(&self, channel: usize, frame: usize) -> T {
                    let index = self.calc_index(channel, frame);
                    T::$read_func(
                        self.buf[index..index + self.bytes_per_sample]
                            .try_into()
                            .unwrap(),
                    )
                }

                implement_size_getters!();
            }

            impl<'a, T> ConverterMut<'a, T> for [< $order $typename >]<&'a mut [u8], T>
            where
                T: Sample<T> + Clone + 'a,
            {
                unsafe fn write_unchecked(&mut self, channel: usize, frame: usize, value: &T) -> bool {
                    let index = self.calc_index(channel, frame);
                    let (value, clipped) = T::$write_func(value);
                    self.buf[index..index + self.bytes_per_sample].clone_from_slice(&value);
                    clipped
                }
            }
        }
    };
}

create_structs!(i16, from_s16_le, to_s16_le, 2, S16LE);
create_structs!(i16, from_s16_be, to_s16_be, 2, S16BE);
create_structs!(i16, from_s24_3_le, to_s24_3_le, 3, S24LE3);
create_structs!(i16, from_s24_3_be, to_s24_3_be, 3, S24BE3);
create_structs!(i16, from_s24_4_le, to_s24_4_le, 4, S24LE4);
create_structs!(i16, from_s24_4_be, to_s24_4_be, 4, S24BE4);
create_structs!(i32, from_s32_le, to_s32_le, 4, S32LE);
create_structs!(i32, from_s32_be, to_s32_be, 4, S32BE);
create_structs!(f32, from_f32_le, to_f32_le, 4, F32LE);
create_structs!(f32, from_f32_be, to_f32_be, 4, F32BE);
create_structs!(f64, from_f64_le, to_f64_le, 8, F64LE);
create_structs!(f64, from_f64_be, to_f64_be, 8, F64BE);

impl_traits!(i16, from_s16_le, to_s16_le, 2, S16LE, Interleaved);
impl_traits!(i16, from_s16_be, to_s16_be, 2, S16BE, Interleaved);
impl_traits!(i16, from_s24_3_le, to_s24_3_le, 3, S24LE3, Interleaved);
impl_traits!(i16, from_s24_3_be, to_s24_3_be, 3, S24BE3, Interleaved);
impl_traits!(i16, from_s24_4_le, to_s24_4_le, 4, S24LE4, Interleaved);
impl_traits!(i16, from_s24_4_be, to_s24_4_be, 4, S24BE4, Interleaved);
impl_traits!(i32, from_s32_le, to_s32_le, 4, S32LE, Interleaved);
impl_traits!(i32, from_s32_be, to_s32_be, 4, S32BE, Interleaved);
impl_traits!(f32, from_f32_le, to_f32_le, 4, F32LE, Interleaved);
impl_traits!(f32, from_f32_be, to_f32_be, 4, F32BE, Interleaved);
impl_traits!(f64, from_f64_le, to_f64_le, 8, F64LE, Interleaved);
impl_traits!(f64, from_f64_be, to_f64_be, 8, F64BE, Interleaved);

impl_traits!(i16, from_s16_le, to_s16_le, 2, S16LE, Sequential);
impl_traits!(i16, from_s16_be, to_s16_be, 2, S16BE, Sequential);
impl_traits!(i16, from_s24_3_le, to_s24_3_le, 3, S24LE3, Sequential);
impl_traits!(i16, from_s24_3_be, to_s24_3_be, 3, S24BE3, Sequential);
impl_traits!(i16, from_s24_4_le, to_s24_4_le, 4, S24LE4, Sequential);
impl_traits!(i16, from_s24_4_be, to_s24_4_be, 4, S24BE4, Sequential);
impl_traits!(i32, from_s32_le, to_s32_le, 4, S32LE, Sequential);
impl_traits!(i32, from_s32_be, to_s32_be, 4, S32BE, Sequential);
impl_traits!(f32, from_f32_le, to_f32_le, 4, F32LE, Sequential);
impl_traits!(f32, from_f32_be, to_f32_be, 4, F32BE, Sequential);
impl_traits!(f64, from_f64_le, to_f64_le, 8, F64LE, Sequential);
impl_traits!(f64, from_f64_be, to_f64_be, 8, F64BE, Sequential);

//   _____         _
//  |_   _|__  ___| |_ ___
//    | |/ _ \/ __| __/ __|
//    | |  __/\__ \ |_\__ \
//    |_|\___||___/\__|___/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_i32() {
        let data: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0, 0, 128, 0, 0, 0, 64, 0, 0, 0, 192, 0, 0, 0, 32, 0, 0, 0, 224,
        ];
        let buffer: InterleavedS32LE<&[u8], f32> = InterleavedS32LE::new(&data, 2, 3).unwrap();
        assert_eq!(buffer.read(0, 0).unwrap(), 0.0);
        assert_eq!(buffer.read(1, 0).unwrap(), -1.0);
        assert_eq!(buffer.read(0, 1).unwrap(), 0.5);
        assert_eq!(buffer.read(1, 1).unwrap(), -0.5);
        assert_eq!(buffer.read(0, 2).unwrap(), 0.25);
        assert_eq!(buffer.read(1, 2).unwrap(), -0.25);
    }

    #[test]
    fn read_i16() {
        let data: Vec<u8> = vec![0, 0, 0, 128, 0, 64, 0, 192, 0, 32, 0, 224];
        let buffer: InterleavedS16LE<&[u8], f32> = InterleavedS16LE::new(&data, 2, 3).unwrap();
        assert_eq!(buffer.read(0, 0).unwrap(), 0.0);
        assert_eq!(buffer.read(1, 0).unwrap(), -1.0);
        assert_eq!(buffer.read(0, 1).unwrap(), 0.5);
        assert_eq!(buffer.read(1, 1).unwrap(), -0.5);
        assert_eq!(buffer.read(0, 2).unwrap(), 0.25);
        assert_eq!(buffer.read(1, 2).unwrap(), -0.25);
    }

    #[test]
    fn write_i32() {
        let expected: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0, 0, 128, 0, 0, 0, 64, 0, 0, 0, 192, 0, 0, 0, 32, 0, 0, 0, 224,
        ];
        let mut data = vec![0; 24];
        let mut buffer: InterleavedS32LE<&mut [u8], f32> =
            InterleavedS32LE::new_mut(&mut data, 2, 3).unwrap();

        buffer.write(0, 0, &0.0).unwrap();
        buffer.write(1, 0, &-1.0).unwrap();
        buffer.write(0, 1, &0.5).unwrap();
        buffer.write(1, 1, &-0.5).unwrap();
        buffer.write(0, 2, &0.25).unwrap();
        buffer.write(1, 2, &-0.25).unwrap();
        assert_eq!(data, expected);
    }

    #[test]
    fn write_i16() {
        let expected: Vec<u8> = vec![0, 0, 0, 128, 0, 64, 0, 192, 0, 32, 0, 224];
        let mut data = vec![0; 12];
        let mut buffer: InterleavedS16LE<&mut [u8], f32> =
            InterleavedS16LE::new_mut(&mut data, 2, 3).unwrap();

        buffer.write(0, 0, &0.0).unwrap();
        buffer.write(1, 0, &-1.0).unwrap();
        buffer.write(0, 1, &0.5).unwrap();
        buffer.write(1, 1, &-0.5).unwrap();
        buffer.write(0, 2, &0.25).unwrap();
        buffer.write(1, 2, &-0.25).unwrap();
        assert_eq!(data, expected);
    }
}
