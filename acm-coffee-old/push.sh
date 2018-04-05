#!/bin/bash

scp -r -i id_rsa . root@$(ssh acm cat coffee_ip):/www/
ssh -i id_rsa root@$(ssh acm cat coffee_ip) 'chmod +x /www/cgi-bin/* ; chmod 777 /www/ ; chmod 777 /www/cgi-bin/'
