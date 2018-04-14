#!/bin/bash

# These control the LCD-CLK pin
echo 120 > /sys/class/gpio/export
echo out > /sys/class/gpio/gpio120/direction
echo 1 > /sys/class/gpio/gpio120/value # 0 for off

sleep 5
echo 0 > /sys/class/gpio/gpio120/value

