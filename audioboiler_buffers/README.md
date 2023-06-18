# audioboiler_buffer

This module is a collection of wrappers that implement the
[audioboiler_traits] traits for various data structures.

## Direct wrappers
The [direct] module provides wrappers for data structures where
the samples are already stored in the desired format.
They implement the [audioboiler_traits::AudioBuffer] and
[audioboiler_traits::AudioBufferMut] traits.
They also imple,ent the [audioboiler_traits::Converter] and
[audioboiler_traits::ConverterMut] (TODO! and rename these)

## Converting wrappers
The [converting] module provides wrappers for slices of raw bytes.
These implement the simpler [audioboiler_traits::Converter] and
[audioboiler_traits::ConverterMut] traits.
When the samples are read or written, the values are converted between
the original format and floating point. 

## License: MIT


