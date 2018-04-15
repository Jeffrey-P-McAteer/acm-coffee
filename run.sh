#!/bin/bash

coffee_procs=$(ps aux | grep coffee | wc -l)
echo "coffee_procs=$coffee_procs"
if [[ "$coffee_procs" -gt 3 ]]; then
  exit 1
fi

cd /opt/coffee
cargo run --release

