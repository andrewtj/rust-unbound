#!/usr/bin/env bash
set -eux -o pipefail
if [[ "$TRAVIS_OS_NAME" == "osx" ]]; then
    brew update
    brew install openssl
    brew install unbound || echo "absorbing brew install error $?"
fi
