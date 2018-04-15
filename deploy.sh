#!/bin/bash

# Start with a flashed "Desktop" CHIP (GUI etc.)
# Default credentials are root/chip and chip/chip
# Serial connection command: sudo screen /dev/ttyACM0

if [[ "$(whoami)" != "root" ]]; then
  echo "Run as root"
  exit 1
fi

if false; then # Over 3g connection
  apt-get update
  apt-get install -y w3m
fi

apt-get install -y vim

# Copy ./auth.sh to /wifi_auth.sh on device

crontab - <<EOF
# Perform auth every 60s
* * * * * /opt/coffee/auth.sh
* * * * * sleep 20; /push_ip_to_cs.sh
* * * * * /opt/coffee/target/release/acm-coffee >>/tmp/log 2>&1
# Clear log file every hour
0 * * * * rm /tmp/log*
EOF

cat > /push_ip_to_cs.sh <<EOF
#!/bin/bash
IP=\$(/sbin/ifconfig wlan0 | grep 'inet addr:' | awk '{print \$2}' | cut -d: -f2)
echo "\$IP" | ssh acm@sirius.cs.odu.edu
# ^ Do that manually once to accept the server cert.
EOF

chmod +x /push_ip_to_cs.sh

mkdir '.ssh' # .ssh/id_coffee on the CS server
cat > .ssh/id_rsa <<EOF
-----BEGIN RSA PRIVATE KEY-----
MIIEowIBAAKCAQEAu0yIKxNguDbZjx4RTmDFchdZAcOvmonj/XuNS6ywErm9VXs6
YKIaeK1KtJ7yxk8ZMgAIMgsdXBxy5vJr1iu9rG0/rRI8bfW3cEzq6uqPowBv3Fne
PWj2Pjeo7m5mGkVbXYLR4jAFhCWdJhnBFMdbeuYjwM2JSCw6PMEsFZVSyrR+a/no
sU/WwPVU4zIkA8dS0b6ieFT6NsjKt0gdA5jGOM64KgwJ+/I9NsDwTYbBKeUEH6/g
Ohimpo3O3EygnsK85UZ/1/BCIhOinQJj6JSEHflYZfUuTbj7gzWpsCLxh8YQ3LXl
0Jogh5bvsgjrapuHip54Brtp7a9TXt3YQr34aQIDAQABAoIBAEsNNr80C5yldfYw
UTT7+AJosqTPWg1t1arcGFlLgF5wiRq4v0K7kinrHrVTv+qRBYKQmrga1g/z4mMC
nw16B44RVOOwHADf5jqcx8GMbjQd17UPWct3xLxXp1yrZkR+qEbHRf0ByyQRwm/j
AAiofdK4Z3k5oxoFhuyTZ2vaowtQKhWVPqPS55tSzwjByKoMP+SAGDzxi/1RsDPz
i9rjGCa3hV6zl2k6sosBYTl9mCJbxfLFifCYxnZ8j10EcN/1k5Fe76Ibv+X/bhV/
5lIy5uOB77XVpG/i8j3+RAYLtZrwPFVECm3JP2pG2US9Bgpd5xGYCyJaG1aUNSfh
ZcOn84ECgYEA7M4JoFEL0gbxDQYvo3MBpfy+emkiYrrwuEAKL9tjj0l8GX3zbWt5
n/MLWxtKU6dojjgs/1gTPbIPKMRzhvFVYFXbjPUCMylulScMD5sT5VEXy6roQkAz
C1YiXuoQsP9Mtc32TfpyPU5EyqNgVQ3ewMdhwtYRMQ5v0I2u9WW4B9kCgYEAynsx
Rb5K3giOAEuqGb38UJ2XZ2JI0YlCxZqe8/yEOWIuK4ruzM92rTcxeL4CRemfwZM2
hdobuSbYLbmucVwGRydKkxvnrFwm8qTXiUOni14zWvVGC2DyBCugrmN1y5sQyvaR
qYCMp11AImPnkEGKS6WF8woHs5suH2S1Xg1UKxECgYEAqH+/V8TznUH1OehB85L9
BEhAnUe8APa2HNTsqrr4L9gBJv55PU2xYIAHf49+puHDQxdz5Umdf1P493A6KDYH
IyJUtAsOwqrjldwP9/bIBG9ceD6nP6UA7Tsf/9ubfuZahi8E6N2hdkAAMRRpknvp
GdGFnabG2tpD8+ktKk5z0ZkCgYAjKyiQu4+XZxb6+ClwVS5Y9jZQ76JEOroNRmDr
ceWpbeMlDvmRO0uapGaEWURdzklAPwiOUSbVjuincIPbDqfMfgeW65beuhbNuFHz
dnvIWjFDUCy9VzZSAR3kjEPufF17Uz5TmY9Uln7IOmADD08s+m/8mbZivMZTD5Ps
5RXEcQKBgCyw8VJCWUSoZySIfeIBemyr1HMXVhdlmaMrIn1MNjiAnU5MYuZZC6r2
LYFYQ97Qmv90RrkMuluEA3/N0sqQ9AfRDCBI3Jb/i5jNk+igtWx/rafYIvKHHuYR
pbohrgMjJuT5CYv7sOlu1uctlfqqnsXpc2QC2BegWUnh1mFs/88L
-----END RSA PRIVATE KEY-----
EOF

chmod 600 .ssh/id_rsa

# Copy id_rsa.pub into .ssh/authroized_keys
# Change PW to C0ff33Stuff

### NETWORKING COMPLETE ###

# Now for the webcam. It requires the gspca_zc3xx module.
#modprobe gspca_zc3xx

apt-get install -y git build-essential pkg-config

git clone --single-branch --branch debian/`uname -r` --depth 1 https://github.com/NextThingCo/CHIP-linux.git /lib/modules/`uname -r`/build/
cd /lib/modules/`uname -r`/build/
cp /boot/config-`uname -r` ./.config
#echo "CONFIG_USB_GSPCA_ZC3XX=m" >> .config
echo "USB_GSPCA_ZC3XX=m" >> .config

make prepare LOCALVERSION=-`uname -r | cut -f2- -d'-'`
make modules_prepare LOCALVERSION=-`uname -r | cut -f2- -d'-'`
make SUBDIRS=scripts/mod LOCALVERSION=-`uname -r | cut -f2- -d'-'`

make SUBDIRS=drivers/media/usb/gspca LOCALVERSION=-`uname -r | cut -f2- -d'-'` modules

cp drivers/media/usb/gspca/gspca_main.ko /lib/modules/`uname -r`/kernel/drivers/media/usb/gspca
cp drivers/media/usb/gspca/gspca_zc3xx.ko /lib/modules/`uname -r`/kernel/drivers/media/usb/gspca

depmod

modprobe gspca_zc3xx

echo gspca_zc3xx >> /etc/modules

apt-get install -y fswebcam
fswebcam -r 640x480 --jpeg 85 -D 1 shot.jpg # Testing

### WEBCAM COMPLETE ###

echo "nobody ALL=(root) NOPASSWD: /www/asroot.sh" > /etc/sudoers.d/passwordless

#apt-get install -y python-opencv

### Rust Install ###

curl -sf -L https://static.rust-lang.org/rustup.sh | sh




