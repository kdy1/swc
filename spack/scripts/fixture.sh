#!/usr/bin/env bash

set -eu

TEST="$(echo "$1" |sed 's/\:\:/\//g' | sed 's/_/-/g')"
export TEST;

cargo test --test fixture pass -- --nocapture