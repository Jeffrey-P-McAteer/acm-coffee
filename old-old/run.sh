#!/bin/bash

# If we are already running, exit
COUNT=$(ps aux | grep run.sh | grep -v grep | wc -l)
echo "[ run.sh ] COUNT = $COUNT"
if [[ "$COUNT" -gt 3 ]]; then
  echo "[ run.sh ] Detected another copy of run.sh, exiting..."
  exit
fi

cd $(dirname $(realpath $0))

# Runs the server


