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


