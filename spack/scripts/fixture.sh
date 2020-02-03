#!/usr/bin/env bash

set -eu

export TEST=$1

cargo test --test fixture pass -- --nocapture