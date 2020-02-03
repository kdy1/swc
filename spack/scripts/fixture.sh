#!/usr/bin/env bash

set -eu

export RUST_LOG='debug'
export CI='1'
export TEST=$1

cargo test --test fixture pass