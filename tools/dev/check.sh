#!/bin/sh
set -eu
cd "$(dirname "$0")/../.."
. ./tools/dev/env.sh
exec python3 tools/dev/test_plan.py "$@"
