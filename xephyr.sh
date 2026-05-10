#!/usr/bin/env bash
set -e

DISPLAY_NUM=:1
LOG=orbitwm.log

pkill Xephyr 2>/dev/null || true
sleep 0.3

Xephyr $DISPLAY_NUM -screen 1280x720 &
XEPHYR_PID=$!

sleep 0.5

DISPLAY=$DISPLAY_NUM cargo run 2>&1 | tee $LOG

kill $XEPHYR_PID 2>/dev/null || true