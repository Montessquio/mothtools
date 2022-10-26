# Mothtools

A collection of command-line tools that can be chained to produce many different results.

## Tools

### Mothlib

Mothlib contains common definitions and manipulations of the Lantern IR format
which internally defines mod behavior.

### Laidlaw

Laidlaw can convert several data formats to and from the Lantern IR format.

Laidlaw can read and write Lantern IR with the following formats:

- JSON
- HJSON
- RON
- Pickle

### Crucible

Crucible is a compiler for the Crucible DSL, a concise language used to describe Cultist Simulator
content quickly and efficiently. Crucible reads, interprets, and applies transformations to code
written in the Crucible language, and outputs it in the Lantern IR format usable by the rest
of the Mothtools suite.

## Under the Hood

### The Lantern Format

Unlike the formats supported by Laidlaw, which serializes Lantern IR to the vanilla CultSim data scheme
(which may not be a trivial conversion), Lantern IR and its serialized format LIR are isomorphic; an LIR
file is item-for-item recreation of the in-program Lantern data structure.

LIR files, which use the `.lir` extension, store their data using strict JSON content. The compressed version,
which uses the `.lirc` extension, is simply a LIR file that has been compressed using Brotli.

Unlike standard JSON, the `.lir` format requires that the first six bytes of every file be
`[4C, 49, 52, 2E, 0D, 0A]` which is ASCII `LIR.\r\n`. The `.lirc` format must start with
`[4C, 49, 43, 2E, 0D, 0A]` which is ASCII `LIRC\r\n`.

### Lantern IR Transmission

When doing I/O over the standard input and output, Mothtools programs will delimit its data stream
using ASCII control sequences. The data producer will transmit a stream as follows:

- ASCII SOH (0x01)
- 4-byte UINT, Size of Header from SOH to STX, inclusive of those two bookending control characters.
- 4-byte UINT, Size of Data from STX to ETX, inclusive of those two bookending control characters.
- 1 byte UINT, SemVer Major protocol version number.
- 1 byte UINT, SemVer Minor protocol version number.
- 2 byte UINT, SemVer Patch protocol version number.
- ASCII STX (0x02)
- A single LIR or LIRC file.
- ASCII ETX (0x03)

The above sequence is repeated until the producer has no more components to send.
When it is finished, the producer will send a single ASCII EOT (0x04) byte.

For example, the full sequence where two LIR files are sent would
look as follows:

- ASCII SOH (0x01)
- 4-byte UINT, Size of Header
- 4-byte UINT, Size of Data from [STX,ETX]
- 4-byte protocol version information
- ASCII STX (0x02)
- A single LIR or LIRC file.
- ASCII ETX (0x03)
- ASCII SOH (0x01)
- 4-byte UINT, Size of Header
- 4-byte UINT, Size of Data from [STX,ETX]
- ASCII STX (0x02)
- A single LIR or LIRC file.
- ASCII ETX (0x03)
- ASCII EOT (0x04)

While the header is of a fixed size in the current version of the specification,
it may change in future versions of Mothlib, so the possibility of a dynamic header
field is reserved to allow greater forward-compatibility. Header metadata is sent
per-transmission unit rather than per-stream, which makes it possible to produce
streams with mixed-version data. It is up to the stream's consumer to handle
version conflicts and interpretation.
