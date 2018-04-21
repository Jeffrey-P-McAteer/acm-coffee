#!/bin/bash

coffee_procs=$(ps aux | grep acm-coffee | grep -v grep | wc -l)
echo "coffee_procs=$coffee_procs"
if [[ "$coffee_procs" -gt 3 ]]; then
  exit 1
fi

cd /opt/coffee
export PATH="/usr/local/bin/:$PATH"
cargo run --release

