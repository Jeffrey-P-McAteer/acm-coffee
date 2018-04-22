#[macro_use]
extern crate lazy_static;
extern crate tiny_http;
extern crate image;
extern crate reqwest;
extern crate time;

use tiny_http::*;
use image::GenericImage;

//use std::ptr;
use std::{thread};
use std::io::prelude::*;
use std::io::{Read, Write};
use std::fs::File;
//use std::path::Path;
use std::env;
use std::sync::Mutex;
//use std::fs::OpenOptions;
//use std::io::{Seek, SeekFrom};
//use std::os::unix::prelude::AsRawFd;

static PRINT_SNAPS : bool = false;
static SHOT_DELAY : u64 = 1100; // ms
static SHARED_SNAP_FILE : &'static str = "/tmp/snap.jpg";
static SHARED_KEY_URL : &'static str = "https://api.keyvalue.xyz/532988dc/coffee-maker-pro";
static ALEXA_DELAY : u64 = 12 * 1000; // ms
static MAX_BREW_S : isize = 10 * 60; // 10 min

lazy_static! {
    static ref SHOT: Mutex<Vec<u8>> = Mutex::new(vec![]);
    static ref METERS: Mutex<Meters> = Mutex::new(construct_meters());
    static ref CURRENT_GROUNDS: Mutex<String> = Mutex::new(String::new());
    static ref CURRENTLY_ON: Mutex<bool> = Mutex::new(false);
    
    static ref START_S: Mutex<i64> = Mutex::new(0);
    static ref LAST_BREW_ON_S: Mutex<i64> = Mutex::new(0);
    
}

fn main() {
  let args: Vec<String> = env::args().collect();
  if args.len() > 1 { // Allow for quick system tests through arguments
    let first = args.get(1).unwrap();
    if first == "test_pot_on" {
      setpot(true);
      return;
    }
    else if first == "test_pot_off" {
      setpot(false);
      return;
    }
    else {
      println!("unknown testing argument '{}'", first);
    }
  }
  
  {
    let mut start_s_ptr = START_S.lock().unwrap();
    *start_s_ptr = timestamp_s();
  }
  
  let webcam_handle    = thread::spawn(|| { webcam_thread();    });
  let webserver_handle = thread::spawn(|| { webserver_thread(); });
  //let alexa_handle     = thread::spawn(|| { alexa_thread();     });
  webcam_handle.join().unwrap();
  webserver_handle.join().unwrap();
  //alexa_handle.join().unwrap();
}

fn webserver_thread() {
  println!("[ webserver ] Spawning web server..");
  let listen_addr = "0.0.0.0:8080";
  let server = Server::http(listen_addr).unwrap();
  println!("[ webserver ] Listening on {}", listen_addr);
  
  let mut v_clone: Vec<u8>;
  
  for request in server.incoming_requests() {
      println!("Request! method: {:?}, url: {:?}", //, headers: {:?}",
          request.method(),
          request.url(),
          //request.headers()
      );
      
      let mut query = String::new();
      let url = format!("{}", request.url());
      let url = if url.contains("?") {
        let first_split = url.split("?").collect::<Vec<&str>>();
        query = format!("{}", first_split.get(1).unwrap());
        query = query.chars().skip(2).take(query.len() - 2).collect();
        query = query.replace("%20", " ")
                     .replace("+", " ");
        format!("{}", first_split.get(0).unwrap() )
      } else {
        url
      };
      let status_txt = match METERS.lock() { Ok(out_meters) => { format!(r#"
CURRENT_GROUNDS: {}
LAST_BREW_ON_S : {}
water_top_x:     {}
water_top_y:     {}
water_bot_x:     {}
water_bot_y:     {}
water_percent:   {}
coffee_top_x:    {}
coffee_top_y:    {}
coffee_bot_x:    {}
coffee_bot_y:    {}
coffee_percent:  {}
"#, CURRENT_GROUNDS.lock().unwrap(), LAST_BREW_ON_S.lock().unwrap(),
  out_meters.water_top_x,
  out_meters.water_top_y,
  out_meters.water_bot_x,
  out_meters.water_bot_y,
  out_meters.water_percent,
  out_meters.coffee_top_x,
  out_meters.coffee_top_y,
  out_meters.coffee_bot_x,
  out_meters.coffee_bot_y,
  out_meters.coffee_percent,
) }
      _ => {
        format!(r#"
CURRENT_GROUNDS: {}
LAST_BREW_ON_S : {}
"#, CURRENT_GROUNDS.lock().unwrap(), LAST_BREW_ON_S.lock().unwrap(),)
      }
    };
      let index_html_string = format!(r#"
<html><head><meta name="viewport" content="width=device-width, initial-scale=1.0"></head><body>
<h1>ACM RFC 2324 Implementation</h1>
<style>
iframe {{
  width: 100%;
  min-height: 220pt;
  border: none;
}}
fieldset {{
  display: inline-block;
  float: left;
  border-width: 1px;
  border-color: #000000;
}}
p {{
  margin: 0;
  float: left;
}}
</style>
<fieldset style="width:440pt;"><legend>Status</legend>
  <iframe src="/status.html"></iframe>
</fieldset>
<fieldset><legend>Controls</legend>
  <form action="/brew" style="float:left;margin-right:12pt;">
    <input type="submit" value="Brew">
  </form>
  <form action="/stop">
    <input type="submit" value="Stop">
  </form>
  <form action="/set-grounds" method="get">
    <p>Grounds flavor: <input name="v"></p>
    <input type="submit" value="Set Grounds">
  </form>
  <form action="/pre-set-coords" method="get">
    <input name="v" type="hidden" style="display:none;" value="null">
    <input type="submit" value="Set Coordinates">
  </form>
</fieldset>
</body>
        "#);
      let bg_color = if *CURRENTLY_ON.lock().unwrap() { "#E55A52" } else { "#81BFFC" };
      let status_html_string = format!(r#"
<meta http-equiv="refresh" content="1;url=/status.html" />
<style>
html, body {{
  background-color: {};
  margin: 0;
  padding: 0;
}}
img, pre {{
  display: inline;
  vertical-align: text-top;
}}
img {{
  margin: 2pt;
  width: 240pt;
}}
</style>
<img src="/snap.jpg" align="left">
<pre>{}</pre>
        "#, bg_color, status_txt);
      let coords_html_string = format!(r#"
<style>
img, pre {{
  display: inline;
  vertical-align: text-top;
}}
img {{
  cursor: crosshair;
}}
</style>
<p>Click the coordinates in the following order: <em>water top, water bottom, coffee top, coffee bottom.</em></p>
<img id="image" src="/snap.jpg" align="left">
<form action="/set-coords">
  <input id="v" name="v" type="hidden" style="display:none;">
  <input type="submit" value="Set Coordinates" style="display:none;">
</form>
<script>
window.water_top = "0,0";
window.water_bot = "0,0";
window.coffee_top = "0,0";
window.coffee_bot = "0,0";

window.coord_count = 0; // 0=water top, 1=water bot, 2=coffee top, 3=coffee bot.

window.addEventListener('DOMContentLoaded', function() {{
  console.log('[ DOMContentLoaded ]');
  document.getElementById('image').onmousedown = function(ev) {{
    var pos_x = ev.offsetX?(ev.offsetX):ev.pageX-document.getElementById("image").offsetLeft;
    var pos_y = ev.offsetY?(ev.offsetY):ev.pageY-document.getElementById("image").offsetTop;
    
    switch (window.coord_count) {{
      case 0:
        window.water_top = pos_x+" "+pos_y;
        break;
      case 1:
        window.water_bot = pos_x+" "+pos_y;
        break;
      case 2:
        window.coffee_top = pos_x+" "+pos_y;
        break;
      case 3:
        window.coffee_bot = pos_x+" "+pos_y;
        break;
    }}
    
    window.coord_count++;
    if (window.coord_count >= 4) {{
      document.getElementById('v').value = window.water_top+"-"+window.water_bot+"-"+window.coffee_top+"-"+window.coffee_bot+"-";
      document.forms[0].submit();
    }}
  }};
}}, true);
</script>
        "#, );
      
      // Response variables
      let mut headers: Vec<Header> = Vec::new();
      let response: Response<&[u8]>;
      
      if url == "/" || url == "/index.html" {
        headers.push(Header::from_bytes(&"Content-Type"[..], &"text/html; charset=utf-8"[..]).unwrap());
        //html_string = read_from_file("/index.htm");
        let html_payload = index_html_string.as_bytes();
        response = Response::new(StatusCode::from(200), headers, &html_payload[..], Some(html_payload.len()), None);
        request.respond(response);
      }
      else if url == "/status.html" {
        headers.push(Header::from_bytes(&"Content-Type"[..], &"text/html; charset=utf-8"[..]).unwrap());
        let html_payload = status_html_string.as_bytes();
        response = Response::new(StatusCode::from(200), headers, &html_payload[..], Some(html_payload.len()), None);
        request.respond(response);
      }
      else if url == "/snap.jpg" {
        headers.push(Header::from_bytes(&"Content-Type"[..], &"image/png"[..]).unwrap());
        match SHOT.lock() {
          Ok(v) => {
            v_clone = v.clone(); // Copy new data into this thread
            response = Response::new(StatusCode::from(200), headers, &v_clone[..], Some(v.len()), None);
            request.respond(response);
          },
          _ => {
            println!("[ Warn ] Could get snap data for http response!");
            let html_payload = "DEVNULL".as_bytes(); // todo make this better
            response = Response::new(StatusCode::from(200), headers, &html_payload[..], Some(html_payload.len()), None);
            request.respond(response);
          }
        }
      }
      else if url == "/pre-set-coords" {
        headers.push(Header::from_bytes(&"Content-Type"[..], &"text/html; charset=utf-8"[..]).unwrap());
        let html_payload = coords_html_string.as_bytes();
        response = Response::new(StatusCode::from(200), headers, &html_payload[..], Some(html_payload.len()), None);
        request.respond(response);
      }
      else if url == "/brew" {
        setpot(true);
        let html_payload = "<meta http-equiv=\"refresh\" content=\"0;URL='/'\" />".as_bytes();
        response = Response::new(StatusCode::from(200), headers, &html_payload[..], Some(html_payload.len()), None);
        request.respond(response);
      }
      else if url == "/stop" {
        setpot(false);
        let html_payload = "<meta http-equiv=\"refresh\" content=\"0;URL='/'\" />".as_bytes();
        response = Response::new(StatusCode::from(200), headers, &html_payload[..], Some(html_payload.len()), None);
        request.respond(response);
      }
      else if url == "/set-grounds" {
        let mut s_pointer = CURRENT_GROUNDS.lock().unwrap();
        *s_pointer = query;
        let html_payload = "<meta http-equiv=\"refresh\" content=\"0;URL='/'\" />".as_bytes();
        response = Response::new(StatusCode::from(200), headers, &html_payload[..], Some(html_payload.len()), None);
        request.respond(response);
      }
      else if url == "/set-coords" {
        update_meters(query);
        let html_payload = "<meta http-equiv=\"refresh\" content=\"0;URL='/'\" />".as_bytes();
        response = Response::new(StatusCode::from(200), headers, &html_payload[..], Some(html_payload.len()), None);
        request.respond(response);
      }
      else { // redir to "/"
        let html_payload = "<meta http-equiv=\"refresh\" content=\"0;URL='/'\" />".as_bytes();
        response = Response::new(StatusCode::from(200), headers, &html_payload[..], Some(html_payload.len()), None);
        request.respond(response);
      }
  }
}

/*fn alexa_thread() {
  println!("[ alexa ] spawning alexa thread...");
  let mut last_body = String::new();
  loop {
    let mut res = reqwest::get(SHARED_KEY_URL).unwrap();
    let mut body = String::new();
    res.read_to_string(&mut body).unwrap();
    
    println!("[ alexa ] BODY: {}", body);
    
    if last_body != body {
      last_body = body;
      if *CURRENTLY_ON.lock().unwrap() {
        setpot(false);
      }
      else {
        setpot(true);
      }
    }
    
    thread::sleep(std::time::Duration::from_millis(ALEXA_DELAY));
  }
}*/

fn webcam_thread() {
  println!("[ webcam ] spawning python proc...");
  std::process::Command::new("sh")
    .args(&["-c", "ps aux | grep python | awk '{print $2}' | xargs kill -9"])
    .output()
    .unwrap();
  let process = match std::process::Command::new("./camera.py").spawn() {
    Ok(p) => p,
    _ => std::process::Command::new("/opt/coffee/camera.py").spawn().unwrap(),
  };
  
  loop {
    if PRINT_SNAPS {
      println!("[ webcam ] Read Snap!");
    }
    
    let mut img = match image::open(SHARED_SNAP_FILE) {
      Ok(i) => i,
      _ => {
        thread::sleep(std::time::Duration::from_millis(SHOT_DELAY));
        continue;
      }
    };
    
    // Modify image
    match METERS.lock() {
      Ok(mut out_meters) => {
        // compute levels with original data
        out_meters.water_percent = percent(&img,
                                           out_meters.water_top_x, out_meters.water_top_y,
                                           out_meters.water_bot_x, out_meters.water_bot_y);
        
        out_meters.coffee_percent = percent(&img,
                                           out_meters.coffee_top_x, out_meters.coffee_top_y,
                                           out_meters.coffee_bot_x, out_meters.coffee_bot_y);
        
        // Draw levels
        let x = (
            (out_meters.water_top_x as f32 * (out_meters.water_percent)) +
            (out_meters.water_bot_x as f32 * (1.0-out_meters.water_percent))
          ) as i32;
        let y = (
            (out_meters.water_top_y as f32 * (out_meters.water_percent)) +
            (out_meters.water_bot_y as f32 * (1.0-out_meters.water_percent))
          ) as i32;
        draw_fat_px(&mut img, x, y);
        
        let x = (
            (out_meters.coffee_top_x as f32 * (out_meters.coffee_percent)) +
            (out_meters.coffee_bot_x as f32 * (1.0-out_meters.coffee_percent))
          ) as i32;
        let y = (
            (out_meters.coffee_top_y as f32 * (out_meters.coffee_percent)) +
            (out_meters.coffee_bot_y as f32 * (1.0-out_meters.coffee_percent))
          ) as i32;
        draw_fat_px(&mut img, x, y);
        
        // Draw guides
        draw_fat_px(&mut img, out_meters.water_top_x, out_meters.water_top_y);
        draw_fat_px(&mut img, out_meters.water_bot_x, out_meters.water_bot_y);
        
        draw_fat_px(&mut img, out_meters.coffee_top_x, out_meters.coffee_top_y);
        draw_fat_px(&mut img, out_meters.coffee_bot_x, out_meters.coffee_bot_y);
        
      },
      _ => {
        println!("[ webcam ] could not lock METERS");
      }
    }
    
    // Dump into shared global var
    let mut data_vec = Vec::<u8>::new();
    img.save(&mut data_vec, image::ImageFormat::JPEG);
    match SHOT.lock() {
      Ok(mut v) => {
        // Remove existing payload
        v.retain(|&_| false);
        //let mut new_v_dat = image.image_encode(".png", vec![]).expect("Error encoding SHOT");
        v.append(&mut data_vec);
      },
      _ => {
        println!("[ webcam ]  Warn ] Could not dump snap!");
      }
    }
    
    thread::sleep(std::time::Duration::from_millis(SHOT_DELAY));
  }
}

fn draw_fat_px(img: &mut image::DynamicImage, x: i32, y: i32) {
  let white = image::Pixel::from_channels(0xff, 0xff, 0xff, 0xff);
  for d_x in -3..3 {
    if x + d_x < 0 || x + d_x > 640 { continue; }
    for d_y in -3..3 {
      if y + d_y < 0 || y + d_y > 480 { continue; }
      img.put_pixel((x + d_x) as u32, (y + d_y) as u32, white);
    }
  }
}

fn percent(img: &image::DynamicImage, x1: i32, y1: i32, /*begin*/ x2: i32, y2: i32 /*end*/) -> f32 {
  let xs = if x1 < x2 { x1 } else { x2 };
  let xl = if x1 > x2 { x1 } else { x2 };
  let ys = if y1 < y2 { y1 } else { y2 };
  let yl = if y1 > y2 { y1 } else { y2 };
  
  let first_pixel = {
    img.get_pixel(xs as u32, ys as u32)
  };
  let last_pixel = {
    img.get_pixel(xl as u32, yl as u32)
  };
  let mut percent = 0.0;
  while percent < 1.0 {
    let x = ( (xs as f32 * (1.0-percent)) + (xl as f32 * (percent)) ) as u32;
    let y = ( (ys as f32 * (1.0-percent)) + (yl as f32 * (percent)) ) as u32;
    let pixel = img.get_pixel(x, y);
    if pixel_closer_to(pixel, last_pixel, first_pixel) {
      break;
    }
    percent += 0.02; // 2%
  }
  return percent;
}

fn pixel_similar(px1: image::Rgba<u8>, px2: image::Rgba<u8>) -> bool {
  let range = 20;
  return (px1.data[0] as i32 - px2.data[0] as i32).abs() < range &&
         (px1.data[1] as i32 - px2.data[1] as i32).abs() < range &&
         (px1.data[2] as i32 - px2.data[2] as i32).abs() < range;
}

// True if 'comp' closer to px1, false if closer to px2
fn pixel_closer_to(comp: image::Rgba<u8>, px1: image::Rgba<u8>, px2: image::Rgba<u8>) -> bool {
  let delta_1 = (px1.data[0] as i32 - comp.data[0] as i32).abs() +
                (px1.data[1] as i32 - comp.data[1] as i32).abs() +
                (px1.data[2] as i32 - comp.data[2] as i32).abs();
  let delta_2 = (px2.data[0] as i32 - comp.data[0] as i32).abs() +
                (px2.data[1] as i32 - comp.data[1] as i32).abs() +
                (px2.data[2] as i32 - comp.data[2] as i32).abs();
  return delta_1 < delta_2;
}

fn setpot(on: bool) {
  write_to_file("/sys/class/gpio/export", "120"); // Likely throw error after writing once
  write_to_file("/sys/class/gpio/gpio120/direction", "out");
  if on {
    if timestamp_s() - (*START_S.lock().unwrap()) < 15 { // Wait 15 seconds before allowing turning on
      println!("[ pot ] not turning on as app started too soon.");
      return;
    }
    println!("[ pot ] ON");
    write_to_file("/sys/class/gpio/gpio120/value", "1");
    let mut s_pointer = CURRENTLY_ON.lock().unwrap();
    *s_pointer = true;
  }
  else {
    println!("[ pot ] OFF");
    write_to_file("/sys/class/gpio/gpio120/value", "0");
    let mut s_pointer = CURRENTLY_ON.lock().unwrap();
    *s_pointer = false;
  }
}

fn write_to_file<S: Into<String>>(file: S, content: S) {
  let file = file.into();
  let content = content.into();
  
  match File::create(&file[..]) {
    Ok(mut f) => {
      match f.write_all(content.as_bytes()) {
        Ok(_) => { },
        _ => {
          //println!("[ Err ] Could not write to {}", &file[..]);
        }
      }
    },
    _ => {
      //println!("[ Err ] Could not open {}", &file[..]);
    }
  }
}

fn read_from_file<S: Into<String>>(file_name: S) -> String {
  let file_name = file_name.into();
  let mut file = File::open(file_name.as_str()).expect("Unable to open the file");
  let mut contents = String::new();
  file.read_to_string(&mut contents).expect("Unable to read the file");
  return contents;
}

fn update_meters(query: String) {
  println!("[ update_meters ] query = {}", query);
  let parts = query.split("-").collect::<Vec<&str>>();
  let mut out_meters = METERS.lock().unwrap();
  out_meters.water_top_x = coord(*parts.get(0).unwrap(), 0);
  out_meters.water_top_y = coord(*parts.get(0).unwrap(), 1);
  
  out_meters.water_bot_x = coord(*parts.get(1).unwrap(), 0);
  out_meters.water_bot_y = coord(*parts.get(1).unwrap(), 1);
  
  out_meters.coffee_top_x = coord(*parts.get(2).unwrap(), 0);
  out_meters.coffee_top_y = coord(*parts.get(2).unwrap(), 1);
  
  out_meters.coffee_bot_x = coord(*parts.get(3).unwrap(), 0);
  out_meters.coffee_bot_y = coord(*parts.get(3).unwrap(), 1);
  
}

fn coord<S: Into<String>>(part: S, num: usize) -> i32 { // 0=x, 1=y
  let part = part.into();
  let parts = part.split(" ").collect::<Vec<&str>>();
  let num = parts.get(num).unwrap().parse::<i32>().unwrap();
  return num;
}

struct Meters {
  water_top_x: i32,
  water_top_y: i32,
  water_bot_x: i32,
  water_bot_y: i32,
  water_percent: f32,
  
  coffee_top_x: i32,
  coffee_top_y: i32,
  coffee_bot_x: i32,
  coffee_bot_y: i32,
  coffee_percent: f32,
}

fn construct_meters() -> Meters {
  Meters {
    water_top_x: 0,
    water_top_y: 0,
    water_bot_x: 0,
    water_bot_y: 0,
    water_percent: 0.0,
    
    coffee_top_x: 0,
    coffee_top_y: 0,
    coffee_bot_x: 0,
    coffee_bot_y: 0,
    coffee_percent: 0.0,
  }
}

fn timestamp_s() -> i64 {
    let timespec = time::get_time();
    timespec.sec
}
