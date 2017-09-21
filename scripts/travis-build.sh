#!/bin/bash

set -ev

function run {
    rm -rf ./target/*/icmp-*

    cargo build --verbose $@
    cargo test --verbose --no-run $@

    TEST_EXECUTABLE=$(find ./target/ -maxdepth 2 -executable -type f -name 'icmp-*')
    echo "Found tests executable at ${TEST_EXECUTABLE}"

    sudo setcap cap_net_raw+ep ${TEST_EXECUTABLE}
    echo "CAP_NET_RAW enabled for ${TEST_EXECUTABLE}"
    eval "${TEST_EXECUTABLE}"

    echo "Test suite finished"
}

if [ "${TRAVIS_RUST_VERSION}" == "nightly" ]
then
    run --features clippy
else
    run
fi
