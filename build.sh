#!/bin/bash

# NO LONGER USED
#if [[ "$1" == "NEWCONTAINER" ]]; then
#  docker build . -t acm-coffee-armv7-unknown-linux-gnueabihf || exit 1
#fi
#cross build --target armv7-unknown-linux-gnueabihf --release || exit 1
#scp -i ./id_rsa ./target/armv7-unknown-linux-gnueabihf/release/acm-coffee root@$(ssh acm cat coffee_ip):/tmp/acm-coffee

IP=$(ssh acm cat coffee_ip)

ssh -i ./id_rsa root@$IP 'mkdir -p /opt/coffee/ ; mkdir -p /opt/coffee/src/'

for file in Cargo.toml run.sh auth.sh; do
  scp -i ./id_rsa $file root@$IP:/opt/coffee/
done

for s_file in src/main.rs; do
  scp -i ./id_rsa $s_file root@$IP:/opt/coffee/src/
done

echo "[ DONE PUSHING ]"

ssh -i ./id_rsa root@$IP "sed -i 's/:8080/:80/g' /opt/coffee/src/main.rs"
ssh -i ./id_rsa root@$IP 'cd /opt/coffee ; cargo run --release'

