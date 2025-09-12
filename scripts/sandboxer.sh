#! /bin/bash

# A wrapper around `sandboxer` to convert command line arguments into environment variables.

# set -x
set -euo pipefail

ARG0="$(basename $0)"

if ! which sandboxer >/dev/null; then
    echo "$ARG0: failed to find 'sandboxer' executable" >&2
    exit 1
fi

usage() {
  echo "usage: $ARG0 [--fs-ro=\"...\"] [--fs-rw=\"...\"] [--tcp-bind=\"...\"] [--tcp-connect=\"...\"] -- <cmd> [args]" >&2
}

export LL_FS_RO=
export LL_FS_RW=
export LL_FS_TCP_BIND=
export LL_FS_TCP_CONNECT=

for ARG in "$@"; do
  case "$ARG" in
    --fs-ro=*)
      export LL_FS_RO="${ARG#*=}"
      shift
      ;;
    --fs-rw=*)
      export LL_FS_RW="${ARG#*=}"
      shift
      ;;
    --tcp-bind=*)
      export LL_TCP_BIND="${ARG#*=}"
      shift
      ;;
    --tcp-connect=*)
      export LL_TCP_CONNECT="${ARG#*=}"
      shift
      ;;
    --)
      shift
      break
      ;;
    *)
      echo "$0: unknown option: $ARG" >&2
      echo >&2
      usage
      exit 1
      ;;
  esac
done

if [[ "$#" -eq 0 ]]; then
    usage
    exit
fi

sandboxer "$@"
