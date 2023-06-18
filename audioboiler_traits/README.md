# audiobuffer_traits

A set of traits for making it easier to work with buffers of audio data.

Audio data can be stored in many different ways,
where both the layout of the data, and the numerical representation can vary.
This crate aims at providing traits that make it easy to write applications
that can handle any data type in any data layout.


## Abstracting the data layout
This module provides several "layers" of traits that add more functionality.
The most basic traits are [Converter] and [ConverterMut]. These provide basic reading and writing.

The next level is the [AudioBuffer] and [AudioBufferMut] traits, that add immutable and mutable
borrowing, and iterators.

The last level is [AudioBufferStats] that is used to calculate some properties of the audio data.
This is implemented for every structure implementing [AudioBuffer] for a numeric type.

By accessing the audio data via the trait methods instead
of indexing the data structure directly,
an application or library becomes independant of the data layout.

## Supporting new data structures
The required trait methods are simple, to make is easy to implement them for
data structures not covered by the existing wrappers in [audioboiler_buffer].

There are default implementations for most methods.
These may be overriden if the wrapped data structure provides a more efficient way
of performing the operation.
For example, the default implementation of `write_from_channel_to_slice()`
simply loops over the elements to copy.
But when the underlying data structure is a sequential slice, then this
can be implemented more efficiently by using [slice::clone_from_slice()].


## License: MIT


