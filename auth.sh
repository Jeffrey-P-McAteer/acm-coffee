#!/bin/bash

# Authenticates device to AccessODU with some credentials

# Credentials MUST be url-encoded
USERNAME='jmcat001'
PASSWORD="$JEFF_ENC_ODU_PW"

REDIR_URL=$(timeout 10 curl -k --silent -D - http://8.8.8.8 | grep 'Location:' | awk '{print $2}' | tr -d '\n' | tr -d '\r')
if [[ "$REDIR_URL" == "" ]]; then
  echo "REDIR_URL is empty, exiting..."
  exit 1
fi

# Get cookies
curl -k -is --cookie-jar /tmp/.portal_payload_cookies "$REDIR_URL"

curl -k --data "buttonClicked=4&redirect_url=http%3A%2F%2Fwww.odu.edu%2Fts%2Faccess%2Fwireless%2Fodu-wireless-networks.html%23redirect%3Dhttp%3A%2F%2Fexample.org%2F&username=$USERNAME&password=$PASSWORD&Submit=Submit" --cookie /tmp/.portal_payload_cookies "https://wirelessauth.odu.edu/login.html"

#<form name="external" id="external" method="post" action="https://wirelessauth.odu.edu/login.html">
#<h1>Login</h1>
#<input type="hidden" name="buttonClicked" size="16" maxlength="15" value="4">
#<input type="hidden" name="redirect_url" size="255" maxlength="255" value="http://www.odu.edu/ts/access/wireless/odu-wireless-networks.html#redirect=http://example.org/">
#<p><label for="username">Username: <span style="color: red">*</span></label><input maxlength="63" size="30" type="text" name="username" /></p>
#<p><label for="password">Password: <span style="color: red">*</span></label><input maxlength="63" size="30" type="password" name="password" /></p>
#<p>Note: <span style="color: red">*</span> Indicates required information</p>
#<p><input class="button" type="submit" value="Submit" name="Submit" /> <input class="button" type="Reset" value="Reset" /></p>
#</form>


