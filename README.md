# AudioBoiler

A set of crates that provide the boilerplate code needed to read and write audio samples
from and to various data structures in various formats.

# Background
Libraries and applications that process audio usually use
a single layout for the audio data internally.
If a project combines libraries that store their audio data differently,
any data passed between them must be converted
by copying the data from a buffer using one layout
to another buffer using the other layout.

## Channels and frames
When audio data has more than one channel is made up of a series of _frames_.
A frame consists of the samples for all channels, belonging to one time point.
For normal stereo, a frame consists of one sample for the left channel
and one for the right, usually in that order.

## Interleaved and sequential
When audio data is stored in a file or in memory,
the data can be ordered in two main ways.
- Keeping all samples for each channel together,
  and storing each channel after the previous.
  This is normally called _sequential_, _non-interleaved_ or _planar_.
  The sample order of a stereo file with 3 frames becomes:
  `L1, L2, L3, R1, R2, R3`
- Keeping all samples for each frame together,
  and storing each frame after the previous.
  This is normally called _interleaved_, and this is how the data in a .wav file is ordered.
  The sample order of a stereo file with 3 frames becomes:
  `L1, R1, L2, R2, L3, R3`
In a more general sense, the same applies when storing
any multi-dimensional array in linear storage such as RAM or a file.
A 2D matrix can then be stored in _row-major_ or _column-major_ order.
The only difference here compared to a general 2D matrix is that the names `row` and `column`
are replaced by the audio-specific `channel` and `frame`.
Using the general notation, _interleaved_ corresponds to _frame-major_ order,
and _sequential_ to _channel-major_ order.

# The modules
## rawsample
Converts audio samples between most formats stored as raw bytes, and floating point values.

## audioboiler_traits
Traits for reading and writing audio data. The aim is to enable projects to
read and write audio data from any buffer, no matter which sample format or
data order the data uses.  

## audioboiler_buffer
Wrappers that implement the [audioboiler_traits] traits for
various data structures.
There are two sets, the [direct] wrappers provide iterators, setters/getters etc
for data structures where the samples are already in the desired format.
The [converting] wrappers provide a simplified api for buffers of raw bytes.
The samples are converted between their original format and floating point
on the fly. 


