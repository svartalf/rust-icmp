#!/bin/bash

set -e

if [ "${TRAVIS_RUST_VERSION}" == "nightly" ]
then
    CARGO_PATH=`whereis -b cargo | cut -d ' ' -f 2`
    cargo build --verbose --features clippy
    sudo ${CARGO_PATH} test --verbose --features clippy
else
    cargo build --verbose
    sudo cargo test --verbose
fi
