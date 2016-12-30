#!/usr/bin/env bash
set -eux -o pipefail
if [[ "$TRAVIS_OS_NAME" == "osx" ]]; then
    export OPENSSL_DIR=$(brew --prefix openssl)
    export UNBOUND_DIR=$(brew --prefix unbound)
fi
./support/build_and_test.pl
./support/run_examples.pl