#!/bin/bash
# Cyberloop CUA controller entrypoint.
#
# Prepares the desktop service from the input save artifact, runs the generic
# Claude battery entrypoint, then exports the desktop state into Flywheel output
# slots before the container exits.
set -e

if ! python3 /app/cua_controller_io.py preload; then
    echo "desktop_unreachable" > /flywheel/termination
    exit 0
fi

set +e
/app/entrypoint.sh
RC=$?

python3 /app/cua_controller_io.py export
EXPORT_RC=$?
if [ "$EXPORT_RC" -ne 0 ]; then
    echo "[cua_controller] export failed (rc=$EXPORT_RC); preserving agent rc=$RC" >&2
fi

exit "$RC"
