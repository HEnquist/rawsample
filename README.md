# RawSample

A library for working with raw audio samples.

Most audio APIs work with buffers of bytes. 
To do anything with the sample values, these raw bytes must be converted to and from numeric types.

This library aims to provide the low level tools for converting most common sample formats from raw bytes to float values. 
Both `f32` and `f64` are supported, as well as both big-endian and little-endian byte order.

Methods are also provided for converting samples between floats (`f32` and `f64`) and integers (`i8`, `u8`, `i16` and `i32`). 

When samples are converted, the amplitude is scaled to fit the new type.
For floats, the values +/- 1.0 are considered full amplitude.
Higher and lower values are possible, but they will be clipped if the sample is converted to an integer type.
For integer types, full amplitude is simply the minimum and maximum possible values of the type.


## Example: 

```rust
use rawsample::{SampleWriter, SampleReader, SampleFormat};

// create a vec of samples
let values = vec![-0.5, -0.25, -0.125, 0.0, 0.125, 0.25, 0.5];

// create a vec to store raw bytes
let mut rawbytes: Vec<u8> = Vec::new();

// write the samples as raw bytes
f64::write_samples(&values, &mut rawbytes, &SampleFormat::S32LE).unwrap();

// create another vec to store the samples after reading back 
let mut values2 = Vec::new();
let mut slice: &[u8] = &rawbytes;

// read the raw bytes back as samples into the new vec 
f64::read_all_samples(&mut slice, &mut values2, &SampleFormat::S32LE).unwrap();
```
