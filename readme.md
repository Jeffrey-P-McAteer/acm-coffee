# ACM-Coffee

An [rfc 2324](https://www.ietf.org/rfc/rfc2324.txt) implementation powered by:

 * [A C.H.I.P](https://getchip.com/pages/chip)
 * [A programmable relay](https://www.adafruit.com/product/2935)
 * A USB Webcam
 * Most important - a regular single-function Coffeepot!

The current server is running in python, but this will be re-written in rust for performance.

(even though the image we process is 640x480 I've seen some embarassing delays just drawing basic objects on it)

## Python Server (deprecated)

Spinning up a local copy of the python server can be done by cd-ing to `./acm-coffee-old/` and running

    python2 -m CGIHTTPServer 8080
    
Then browse to [http://localhost:8080/cgi-bin/coffeepot.py](http://localhost:8080/cgi-bin/coffeepot.py).

(It's not very mistake friendly, one of the reasons a rewrite is in order)

## Rust server

If you have `cargo` installed run the server with

    cargo run --release

## Easy SSH Access

If you have an acm .ssh/config entry a one-liner is:

    ssh -i id_rsa root@$(ssh acm cat coffee_ip)

If you don't have access to the acm cs account, get the IP of the C.H.I.P server by looking at the src=<...> attribute of the iframe on [http://www.cs.odu.edu/~acm/cgi-bin/coffeepot.cgi](http://www.cs.odu.edu/~acm/cgi-bin/coffeepot.cgi). You will need to be on ODU Wifi for this to work (It's a local address, something else which needs to change).

Then run

    ssh -i id_rsa root@<IP>

Yes, I'm effectively giving everyone root access to the coffee. No, I don't think this is particularly evil, the box has a hard reset button and all the important data is stored here under git revision.

## Important files

`./deploy.sh` contains all commands needed to go from a base C.H.I.P to the current environment (installs packages, sets up cron, etc).

`./asroot.sh` used to be important but is no longer

`./auth.sh` does wifi authentication to AccessODU, you will have to replace $JEFF_ENC_ODU_PW with your PW or set JEFF_ENC_ODU_PW when running the process.

## Closing

Lots of stuff is going to change as soon as final projects get finished, I'm pushing this repo now for curious h4x0rs who want to try things.


