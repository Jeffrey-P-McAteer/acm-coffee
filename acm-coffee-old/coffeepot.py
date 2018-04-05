#!/usr/bin/env python2

# An implementation of https://www.ietf.org/rfc/rfc2324.txt
# for use in environments where CGI is provided (Like ODU's cs server).
# This will generally live at coffee://cs.odu.edu/~acm/coffeepot

import cgi, os, sys, subprocess, time, cv2, math
import random
from datetime import datetime

# Range for color comparison
COLOR_RANGE = 10
# rgba tuples for known colors
POT_WHITE_COLOR = (0x92, 0x95, 0x82, 0xff)
POT_EMPTY_WATER_COLOR = (0x60, 0x5c, 0x51, 0xff)
# Position information
Bars = {
  "WATER_LEVEL": {
    "xtop": 104,
    "ytop": 5,
    "xbot": 18,
    "ybot": 274,
    #"BG_COLOR":   (0x88, 0x90, 0x92, 0x00), # Does this really matter?
    "FILL_COLOR": (0x6e, 0x6a, 0x69, 0x00),
  },
  "COFFEE_LEVEL": {
    "xtop": 245,
    "ytop": 300,
    "xbot": 220,
    "ybot": 426,
    #"BG_COLOR":   (0x88, 0x90, 0x92, 0x00), # Does this really matter?
    "FILL_COLOR": (0x00, 0x00, 0x00, 0x00),
  }
}

SWD = os.path.dirname( os.path.realpath(__file__) ) # Script working dir
# str(subprocess.check_output(["whoami"])) # 'nobody'

def file2str(file):
  data = ""
  with open(file, 'r') as myfile:
    data = myfile.read()
  return data

def color_diff(color1, color2): # TODO replace get_bar_percent with step color delta (compare last/initial color to current, see if jump above threshold)
  s = 0.0
  for i in range(0, 2):
    s += math.fabs(color1[i] - color2[i])
  return s

def color_equals(color1, color2, variance=COLOR_RANGE):
  diff = [0, 0, 0]
  for chan in range(0,3):
    diff[chan] = color1[chan] - color2[chan]
  return math.fabs(diff[0]) < variance and math.fabs(diff[1]) < variance and math.fabs(diff[2]) < variance

def dist(x1, y1, x2, y2):
  return math.sqrt( ((x1 - x2)**2) + ((y1 - y2)**2) )

def get_bar_percent(img, bar): # from 0.0 to 100.0
  x = bar["xtop"]
  y = bar["ytop"]
  DELTAS = 20
  length = dist(bar["xtop"], bar["xbot"], bar["ytop"], bar["ybot"])
  x_step = (bar["xtop"] - bar["xbot"]) / length
  y_step = (bar["ytop"] - bar["ybot"]) / length
  
  marker_x = x
  marker_y = y
  while y < bar["ybot"] and (y >= 0 and y < 400 and x >= 0 and x < 400): # Safeguards
    x -= x_step
    y -= y_step
    marker_x = x
    marker_y = y
    if color_equals(img[y,x], bar["FILL_COLOR"]):
      break
  
  marker_len = dist(bar["xtop"], marker_x, bar["ytop"], marker_y)
  
  wrong_percent = (marker_len / length) * 100.0
  return (100.0 - wrong_percent, (marker_x, marker_y))

if len(sys.argv) > 1 and sys.argv[1] == "--server":
  os.system("python3 -m http.server --bind 0.0.0.0 --cgi 8080")
  sys.exit(0)

if len(sys.argv) > 1 and sys.argv[1] == "--cam-process":
  camproc_running = len( str(subprocess.check_output(["ps", "aux"])).split("--cam-process") )
  if camproc_running > 4: # '4' for if we run manually
    print "[ --cam-process ] Process already running, exiting... (camproc_running="+str(camproc_running)+")"
    sys.exit(1)
  print "[ --cam-process ] aquiring camera..."
  cam = cv2.VideoCapture(0)
  while True:
    ret, frame = cam.read()
    # Post process
    
    height, width, channels = frame.shape # frame[row, col] frame[y, x] (blue, green, red)
    
    # Dump
    water_lv, water_coords = get_bar_percent(frame, Bars["WATER_LEVEL"])
    water_x, water_y = water_coords
    coffee_lv, coffee_coords = get_bar_percent(frame, Bars["COFFEE_LEVEL"])
    coffee_x, coffee_y = coffee_coords
    with open(SWD+"/media/shot.dat", "w") as text_file:
      text_file.write("""
Water fill percent:  %d%%
Coffee fill percent: %d%%
""" % (water_lv, coffee_lv))
    # Write debugging lines etc
    if True:
      # Top
      frame[Bars["WATER_LEVEL"]["ytop"]-1:Bars["WATER_LEVEL"]["ytop"]+1, Bars["WATER_LEVEL"]["xtop"]-5:Bars["WATER_LEVEL"]["xtop"]+5] = (0x0, 0x0, 0xff)
      frame[Bars["COFFEE_LEVEL"]["ytop"]-1:Bars["COFFEE_LEVEL"]["ytop"]+1, Bars["COFFEE_LEVEL"]["xtop"]-5:Bars["COFFEE_LEVEL"]["xtop"]+5] = (0x0, 0x0, 0xff)
      # Bot
      frame[Bars["WATER_LEVEL"]["ybot"]-1:Bars["WATER_LEVEL"]["ybot"]+1, Bars["WATER_LEVEL"]["xbot"]-5:Bars["WATER_LEVEL"]["xbot"]+5] = (0x0, 0x0, 0xff)
      frame[Bars["COFFEE_LEVEL"]["ybot"]-1:Bars["COFFEE_LEVEL"]["ybot"]+1, Bars["COFFEE_LEVEL"]["xbot"]-5:Bars["COFFEE_LEVEL"]["xbot"]+5] = (0x0, 0x0, 0xff)
      # Level
      frame[water_y-2:water_y+2, water_x-5:water_x+5] = (0x0, 0xff, 0x0)
      frame[coffee_y-2:coffee_y+2, coffee_x-5:coffee_x+5] = (0x0, 0xff, 0x00)
      
    print "Saving shot to", SWD+"/media/shot.jpg"
    cv2.imwrite(SWD+"/media/shot.jpg", frame)
    print "Saving metadata to", SWD+"/media/shot.dat"
    time.sleep(2)
  sys.exit(0)


def boring_headers(finish): # Writes \n\n if finish is true, which ends the headers and begins content.
  print "Content-type: text/html"
  TIMESTAMP = datetime.now().strftime("%a, %d %b %Y %X %Z EST")
  print "Date:", TIMESTAMP # Wed, 28 Mar 2018 16:09:07 GMT
  print "Server: odu-acm-nicknacks-python2"
  if finish: print ""

def brew():
  print "TODO"

def get():
  print "HTTP/1.1 418 I'm a Coffeepot"
  boring_headers(True)
  get_arguments = cgi.FieldStorage()
  if "status" in get_arguments:
    print """<!doctype HTML>
<html>
  <head>
    <title>ACM Coffeepot Status</title>
    <meta http-equiv="refresh" content="2">
  </head>
  <body>
    <img src="/media/shot.jpg">
    <pre>%s</pre>
  </body>
</html>
""" % file2str(SWD+"/media/shot.dat")
  else:
    print """<!doctype HTML>
<html>
  <head>
    <title>ACM Coffeepot Redirect</title>
    <meta http-equiv="refresh" content="0;URL='/'" />
  </head>
  <body>
    <h1>Redirecting...</h1>
  </body>
</html>
"""


# Main code

METHOD = os.environ["REQUEST_METHOD"].lower() if "REQUEST_METHOD" in os.environ else "get"

handlers = {
  "get": get,
  "brew": brew
}

handlers[METHOD]() if METHOD in handlers else get()


