#!/bin/bash

# If we are already running, exit
COUNT=$(ps aux | grep run.sh | grep -v grep | wc -l)
echo "[ run.sh ] COUNT = $COUNT"
if [[ "$COUNT" -gt 3 ]]; then
  echo "[ run.sh ] Detected another copy of run.sh, exiting..."
  exit
fi

cd $(dirname $(realpath $0))
echo "[ run.sh ] PWD = $(pwd)"

rm cgi-bin/coffeepot.py
ln -s `pwd`/coffeepot.py cgi-bin/coffeepot.py

chmod +rw /dev/video0 # So 'nobody' can read
usermod -a -G video nobody

find . -exec chmod +rw {} \; # Set read + write on ALL files under /www/

python2 -m CGIHTTPServer 8080

