#!/bin/bash
echo "Building examples for Raspberry Pi Zero W..."
echo ""
echo "=> linux"
cargo build --release --example linux --target=arm-unknown-linux-gnueabihf
