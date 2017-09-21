#!/bin/bash

set -ev

CARGO_PATH=`whereis -b cargo | cut -d ' ' -f 3`
echo "whereis cargo result: `whereis -b cargo`"
echo "${CARGO_PATH} will be used for sudo-based tests"

if [ "${TRAVIS_RUST_VERSION}" == "nightly" ]
then
    cargo build --verbose --features clippy
    sudo ${CARGO_PATH} test --verbose --features clippy
else
    cargo build --verbose
    sudo ${CARGO_PATH} cargo test --verbose
fi
