#!/bin/sh
set -eu
cd "$(dirname "$0")/../.."
. ./tools/dev/env.sh
cargo xwin test --no-run --locked --target x86_64-pc-windows-msvc -p azeroth-coop-native --lib
