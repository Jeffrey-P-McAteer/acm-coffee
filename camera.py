#!/usr/bin/env python2

# An implementation of https://www.ietf.org/rfc/rfc2324.txt
# for use in environments where CGI is provided (Like ODU's cs server).
# This will generally live at coffee://cs.odu.edu/~acm/coffeepot

import os, sys, subprocess, time, cv2, math
import random
from datetime import datetime
import mmap

SHARED_SNAP_FILE = "/tmp/snap.jpg"
SHOT_DELAY = 1 # seconds

print "[ camera.py ] aquiring camera..."
cam = cv2.VideoCapture(0)
while True:
  ret, frame = cam.read()
  height, width, channels = frame.shape # frame[row, col] frame[y, x] (blue, green, red)
  
  cv2.imwrite(SHARED_SNAP_FILE, frame)
  print "[ camera.py ] Snap!"
  time.sleep(SHOT_DELAY)

