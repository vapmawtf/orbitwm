#!/usr/bin/env bash
set -e

DISPLAY=:1 kitty &
sleep 0.2
DISPLAY=:1 kitty &

wait