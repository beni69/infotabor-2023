#!/bin/sh
set -e

label="MICROBIT"
file=$1

udisksctl mount -b /dev/disk/by-label/$label
# cp $1 /run/media/$USER/$label/
rsync -vh --progress $1 /run/media/$USER/$label/
ls -lAh /run/media/$USER/$label/
udisksctl unmount -b /dev/disk/by-label/$label
