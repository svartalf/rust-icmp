#!/bin/bash

set -ev

function run {
    rm -rf ./target/*/icmp-*

    cargo build --verbose $@
    cargo test --verbose --no-run $@

    if [ "${TRAVIS_OS_NAME}" == "osx" ]
    then
        FIND_FLAGS="-perm +0111"
    else
        FIND_FLAGS="-executable"  # Fallback to GNU find syntax
    fi
    TEST_EXECUTABLE=$(find ./target/ -maxdepth 2 ${FIND_FLAGS} -type f -name 'icmp-*')
    echo "Found tests executable at ${TEST_EXECUTABLE}"

    eval "sudo ${TEST_EXECUTABLE}"

    echo "Test suite finished"
}

if [ "${TRAVIS_RUST_VERSION}" == "nightly" ]
then
    run --features clippy
else
    run
fi
