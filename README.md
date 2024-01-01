# PIE - Pixel Indexed Encoding
> Version 1.0.1

## Description

This lossless image format only optionally stores colors in the file.
It is designed to be used in conjunction with a palette from which
colours can be sampled by the decoder.

Using an external palette reduces uncompressed image size by 75%
assuming a four channel format like RGBA, or 60% assuming a 3
channel format like RGB without alpha.

Using an internal palette will increase the size depending on the
palette, but still generally be smaller than other formats like PNG
for pixel art.

## Comparison

In the images/ folder you will find randomly selected .png pixel art
images from lospec.org as well as converted .pie files. If any of
these images are your and you want it removed, please create an issue.

| File | Size Difference |
| --- | --- |
| a-strawberry-dude-509249.pie   | 77.00% the size of the png version
| cubikism-023391.pie            | 81.00% ..
| dune-portraits-787893.pie      | 74.00% ..
| goblin-slayer-808592.pie       | 63.00% ..
| khorne-berserker-509756.pie    | 50.00% ..
| snowfighter-844418.pie         | 64.00% ..

Usage & docs: see https://github.com/Falconerd/pie/blob/master/pie.h
