#!/bin/bash

IP=$(ssh acm cat coffee_ip)

#cargo build --release || exit 1
cargo clean

ssh -i ./id_rsa root@$IP 'mkdir -p /opt/coffee/'

mv .git /tmp/.git
scp -r -i ./id_rsa ./ root@$IP:/opt/coffee/
mv /tmp/.git ./.git

echo "[ DONE PUSHING ] Expect at least 6 minute compile time"

time ssh -i ./id_rsa root@$IP 'cd /opt/coffee ; cargo build --release && killall acm-coffee # ( killall acm-coffee ; sleep 1 ; cargo run --release )'

