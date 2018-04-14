#!/bin/bash

IP=$(ssh acm cat coffee_ip)

ssh -i ./id_rsa root@$IP 'mkdir -p /opt/coffee/'

scp -i ./id_rsa ./run.sh root@$IP:/opt/coffee/

echo "[ DONE PUSHING ]"

ssh -i ./id_rsa root@$IP '/opt/coffee/run.sh'
