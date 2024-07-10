## catmidi

A simple CLI tool to read/write MIDI ports via stdio.

### Examples

Requesting pattern data from Behringer TD-3:

```
$ echo 'F0 00 20 32 00 01 0A 77 00 00 F7' | catmidi rw 'TD-3' 'TD-3'
f0 00 20 32 00 01 0a 78 00 00 00 00 02 02 01 08 01 08 01 08 01 08 01 08 01 08 01 08 01 08 01 08 01 08 01 08 01 08 01 08 01 08 01 08 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 04 00 00 00 0f 00 00 00 0e 00 00 f7
```

Connecting two MIDI ports:

``` 
catmidi r 'VirtualMidi Port1' | catmidi w 'TD-3'
```