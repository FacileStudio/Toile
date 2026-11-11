#!/usr/bin/env sh
set -eu

# Regenerates the PNG icons, favicon.ico and the Open Graph image from static/icon.svg and
# static/og.svg. Needs ImageMagick 7 (`magick`) and the DejaVu Sans font.

cd "$(git rev-parse --show-toplevel)/apps/client/static"

for size in 192 512; do
  magick -background none -density 300 icon.svg -resize "${size}x${size}" "icon-${size}.png"
done
magick -background none -density 300 icon.svg -resize 180x180 apple-touch-icon.png
magick -background none -density 300 icon.svg -define icon:auto-resize=32,16 favicon.ico
magick -density 96 og.svg og.png
