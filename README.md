Work in progress. A tool to facilitate use of 'chord charts'

This shall consist of several parts (in order of propority):

- A language to quickly describe the chord charts and song lists
    - Songs are divided into sections
    - Can have lyrics and chord symbols
    - Chords are semantictly parsed to a degree
    - (Still looking for a compact representation of rhythm)
    - May allow for ABC notation lines in the future
- A Static Site Generator (non-configurable)
    - Outputs html and binary files for each song
    - Uses the binary files to generate song lists
    - The songlist shall be able to specify:
        - The target tonality
        - Song order of sections
    - Make aggregates for
        - Indices of songs and songlists
        - Text extracts for client-side searches
    - Make an archive
- A website template for the above functionality

Current planned folder structure
- site
    - data
        - lst (Song lists)
        - shb (Song source files)
        - cbor (Serialized files)
    - song
        - (generated songs)
    - list
        - (generated lists)
    - css / js / etc
    


Currently only the song language is implemented.
It is supposed to be able to run and compile from a mobile device.

I'm new to rust, the code may be horrendous.
This is actually the 3rd implementation, don't ask.