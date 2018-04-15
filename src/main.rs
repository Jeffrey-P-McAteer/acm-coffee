#[macro_use]
extern crate lazy_static;
extern crate tiny_http;
extern crate image;

use tiny_http::*;

use std::ptr;
use std::{thread, time, fs};
use std::io::prelude::*;
use std::io::{Read, Write};
use std::fs::File;
use std::path::Path;
use std::env;
use std::sync::Mutex;
use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom};
use std::os::unix::prelude::AsRawFd;

static SHOT_DELAY : u64 = 1100; // ms
static SHARED_SNAP_FILE : &'static str = "/tmp/snap.jpg";

lazy_static! {
    static ref SHOT: Mutex<Vec<u8>> = Mutex::new(vec![]);
    static ref METERS: Meters = construct_meters();
    static ref CURRENT_GROUNDS: Mutex<String> = Mutex::new(String::new());
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
  
  let webcam_handle    = thread::spawn(|| { webcam_thread();    });
  let webserver_handle = thread::spawn(|| { webserver_thread(); });
  webcam_handle.join().unwrap();
  webserver_handle.join().unwrap();
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
      
      let status_txt = format!(r#"
Currrent Grounds: {}
"#, CURRENT_GROUNDS.lock().unwrap());
      
      let index_html_string = format!(r#"
<h1>ACM RFC 2324 Implementation</h1>
<style>
iframe {{
  width: 100%;
  min-height: 400px;
  border: none;
}}
</style>
<iframe src="/status.html"></iframe>
<hr>
<form action="/brew">
  <input type="submit" value="Brew">
</form>
<form action="/stop">
  <input type="submit" value="Stop">
</form>
<form action="/set-grounds" method="get">
  <p>Grounds flavor: <input name="v"></p>
  <input type="submit" value="Set Grounds">
</form>
        "#);
      let status_html_string = format!(r#"
<meta http-equiv="refresh" content="1;url=/status.html" />
<style>
img, pre {{
  display: inline;
  vertical-align: text-top;
}}
</style>
<img src="/snap.png" width="420px">
<pre>{}</pre>
        "#, status_txt);
      
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
      else if url == "/snap.png" {
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
      else { // redir to "/"
        let html_payload = "<meta http-equiv=\"refresh\" content=\"0;URL='/'\" />".as_bytes();
        response = Response::new(StatusCode::from(200), headers, &html_payload[..], Some(html_payload.len()), None);
        request.respond(response);
      }
  }
}

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
    println!("[ webcam ] Read Snap!");
    
    let img = match image::open(SHARED_SNAP_FILE) {
      Ok(i) => i,
      _ => {
        thread::sleep(time::Duration::from_millis(SHOT_DELAY));
        continue;
      }
    };
    
    let mut data_vec = Vec::<u8>::new();
    img.save(&mut data_vec, image::ImageFormat::PNG);
    
    // Dump into shared global var
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
    
    thread::sleep(time::Duration::from_millis(SHOT_DELAY));
  }
}

fn setpot(on: bool) {
  write_to_file("/sys/class/gpio/export", "120"); // Likely throw error after writing once
  write_to_file("/sys/class/gpio/gpio120/direction", "out");
  if on {
    println!("[ pot ] ON");
    write_to_file("/sys/class/gpio/gpio120/value", "1");
  }
  else {
    println!("[ pot ] OFF");
    write_to_file("/sys/class/gpio/gpio120/value", "0");
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

fn update_meters() {
  
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