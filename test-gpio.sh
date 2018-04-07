#!/bin/bash

# Tests pushing voltage out of a GPIO pin C0ff33Stuff

# wget https://gist.githubusercontent.com/jefflarkin/b2fcec3817ea5d85288f/raw/30717f7ad7c59eb9f80908687e4ec7ae7d150c8c/gpio.sh
# sed -i 's/408 +/1020 +/g' gpio.sh
# source gpio.sh

# gpio_enable 0
# gpio_mode 0 out
# gpio_write 0 0 # Backwards, for when GPIO is _ground_
# # Same thing looped
# for ((i=0; i<100; i++)) ; do gpio_write 0 $((i%2)) ; sleep 1 ; done

# # BELOW IS OLD

# # Pins XIO-P0 to P7 linearly map to
# #    gpio408 to gpio415    on kernel 4.3
# #    gpio1016 to gpio1023  on kernel 4.4.11
# #    gpio1013 to gpio1020  on kernel 4.4.13-ntc-mlc

# echo out > /sys/class/gpio/gpio1020/direction
# cat /sys/class/gpio/gpio1020/direction
# echo 1 > /sys/class/gpio/gpio1020/value
# sleep 5
# echo 0 > /sys/class/gpio/gpio1020/value

# These control the LCD-CLK pin
echo 120 > /sys/class/gpio/export
echo out > /sys/class/gpio/gpio120/direction
echo 1 > /sys/class/gpio/gpio120/value # 0 for off
