#!/bin/sh
set -e

label="RPI-RP2"
file=./target/thumbv6m-none-eabi/release/tvremote-relay

udisksctl mount -b /dev/disk/by-label/$label
elf2uf2-rs -d target/thumbv6m-none-eabi/release/tvremote-relay
udisksctl unmount -b /dev/disk/by-label/$label

