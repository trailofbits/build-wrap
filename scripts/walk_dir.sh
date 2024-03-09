#! /bin/bash

set -x
set -euo pipefail

if [[ $# -ne 0 ]]; then
    echo "$0: expect no arguments" >&2
    exit 1
fi

WITH_CC="--config=target.'cfg(all())'.linker='cc'"
WITH_BUILD_WRAP="--config=target.'cfg(all())'.linker='build-wrap'"

DEFAULT_TOOLCHAIN="$(rustup default | grep -o '^[^ ]*')"

echo -n > successes.txt
echo -n > failures.txt

DIR="$PWD"

find . -name build.rs |
while read X; do
    Y="$(dirname "$X")"

    pushd "$Y"

    while true; do
        if ! (rustup which rustc | grep -w "$DEFAULT_TOOLCHAIN"); then
            break
        fi

        if ! cargo clean "$WITH_CC"; then
            break
        fi

        if ! cargo build "$WITH_CC"; then
            break
        fi

        cargo clean "$WITH_CC"

        if cargo build "$WITH_BUILD_WRAP"; then
            echo "$Y" >> "$DIR"/successes.txt
            break
        fi

        echo "$Y" >> "$DIR"/failures.txt

        break
    done

    popd
done

cat failures.txt
