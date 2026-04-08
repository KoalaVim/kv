#!/bin/bash
set -e

LAZY_DIR="$HOME/.local/share/kvim-envs/main/lazy"

# First-run: if plugins haven't been installed yet, sync them
if [ ! -d "$LAZY_DIR" ] || [ -z "$(ls -A "$LAZY_DIR" 2>/dev/null)" ]; then
    echo "First run detected — installing plugins (this may take a minute)..."
    NVIM_APPNAME=kvim-envs/main nvim --headless "+Lazy! sync" +qa 2>/dev/null || true
    echo "Done."
fi

exec kv "$@"
