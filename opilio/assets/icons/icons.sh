#!/bin/bash

# specify the input file
input_file="icon.png"

# specify the sizes
sizes=(16 22 24 32 48 64 72 96 128 192 256)

# loop over each size
for size in "${sizes[@]}"; do
    # use ImageMagick to resize the image
    convert $input_file -resize "${size}x${size}" "opilio_${size}x${size}.png"
done
