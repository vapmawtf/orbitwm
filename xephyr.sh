#!/usr/bin/env bash
set -e

DISPLAY_NUM=:1
LOG=orbitwm.log
XAUTH=/tmp/orbitwm-xephyr.auth

pkill Xephyr 2>/dev/null || true
sleep 0.3

rm -f "$XAUTH"
touch "$XAUTH"
xauth -f "$XAUTH" add "$DISPLAY_NUM" . "$(mcookie)" >/dev/null 2>&1 || true

Xephyr $DISPLAY_NUM -screen 1280x720 -auth "$XAUTH" &
XEPHYR_PID=$!

sleep 0.5

XAUTHORITY=$XAUTH DISPLAY=$DISPLAY_NUM DEBUG=1 cargo run 2>&1 | tee $LOG

kill $XEPHYR_PID 2>/dev/null || true