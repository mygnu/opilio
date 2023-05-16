#!/bin/bash

cargo build --release
cargo deb -p opilio
