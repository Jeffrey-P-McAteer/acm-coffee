#!/bin/sh

# Lives at http://www.cs.odu.edu/~acm/cgi-bin/coffeepot.cgi
# Prints <iframe> page with src=~/coffee_ip

echo "Content-type: text/html"
echo ""
IP=$(cat /home/acm/coffee_ip | tr -d '\n')
echo "<style>body, html, iframe { margin: 0; padding: 0; }</style><iframe src='http://$IP:8080/' style='border:none;width:100%;height:100%;min-height:900px;'></iframe>"

