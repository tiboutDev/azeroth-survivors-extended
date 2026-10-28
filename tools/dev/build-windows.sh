#!/bin/sh
set -eu
while [ "$#" -gt 0 ]; do
    case "$1" in
        --build-only) shift ;;
        *) echo "Unknown argument: $1" >&2; exit 2 ;;
    esac
done
cd "$(dirname "$0")/../.."
. ./tools/dev/env.sh
cargo xwin build --release --locked --target x86_64-pc-windows-msvc -p azeroth-coop-native
