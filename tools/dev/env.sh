#!/bin/sh
# Source after changing to the project root. Keep caches on the Linux disk.
# Explicit/private checkout caches are optional; never guess a sibling workspace.
if [ -z "${AZCOOP_TOOLS_DIR:-}" ] && [ -d "$PWD/.dev-tools/cargo" ]; then
    AZCOOP_TOOLS_DIR="$PWD/.dev-tools"
fi
if [ -n "${AZCOOP_TOOLS_DIR:-}" ]; then
    export CARGO_HOME="${CARGO_HOME:-$AZCOOP_TOOLS_DIR/cargo}"
    export RUSTUP_HOME="${RUSTUP_HOME:-$AZCOOP_TOOLS_DIR/rustup}"
    export XWIN_CACHE_DIR="${XWIN_CACHE_DIR:-$AZCOOP_TOOLS_DIR/xwin}"
else
    export CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}"
    export RUSTUP_HOME="${RUSTUP_HOME:-$HOME/.rustup}"
fi
export PATH="$CARGO_HOME/bin:/usr/lib/llvm-18/bin:$PATH"

# Optional configured SDK overlay for case-insensitive header lookup.
if [ -n "${AZCOOP_TOOLS_DIR:-}" ] && [ -f "$AZCOOP_TOOLS_DIR/windows-sdk-vfs.json" ]; then
    export CFLAGS="${CFLAGS:-} -Xclang -ivfsoverlay -Xclang $AZCOOP_TOOLS_DIR/windows-sdk-vfs.json"
    export CXXFLAGS="${CXXFLAGS:-} -Xclang -ivfsoverlay -Xclang $AZCOOP_TOOLS_DIR/windows-sdk-vfs.json"
fi

# Keep disposable builds bounded on the WSL host disk. Release symbols are
# controlled independently by Cargo.toml. Explicit diagnostic overrides win.
export CARGO_INCREMENTAL="${CARGO_INCREMENTAL:-0}"
export CARGO_PROFILE_DEV_DEBUG="${CARGO_PROFILE_DEV_DEBUG:-0}"
export CARGO_PROFILE_TEST_DEBUG="${CARGO_PROFILE_TEST_DEBUG:-0}"
