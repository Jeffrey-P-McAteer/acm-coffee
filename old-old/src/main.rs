#[macro_use]
extern crate lazy_static;
extern crate tiny_http;
extern crate cv;

use tiny_http::*;
use cv::highgui::*;
use cv::videoio::VideoCapture;

use std::{thread, time, fs};
use std::io::prelude::*;
use std::io::Write;
use std::io::Read;
use std::fs::File;
use std::path::Path;
use std::env;
use std::sync::Mutex;

static SHOT_DELAY : u64 = 1200; // ms

lazy_static! {
    static ref SHOT: Mutex<Vec<u8>> = Mutex::new(vec![]);
    static ref METERS: Meters = construct_meters();
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
  
  println!("Spawning threads...");
  let webcam_handle    = thread::spawn(|| { webcam_thread();    });
  let webserver_handle = thread::spawn(|| { webserver_thread(); });
  webcam_handle.join().unwrap();
  webserver_handle.join().unwrap();
}

fn webserver_thread() {
  let listen_addr = "0.0.0.0:8080";
  let server = Server::http(listen_addr).unwrap();
  println!("Listening on {}", listen_addr);
  
  let mut v_clone: Vec<u8>;
  
  for request in server.incoming_requests() {
      println!("Request! method: {:?}, url: {:?}", //, headers: {:?}",
          request.method(),
          request.url(),
          //request.headers()
      );
      
      let url = format!("{}", request.url());
      
      // Response variables
      let mut headers: Vec<Header> = Vec::new();
      let response: Response<&[u8]>;
      
      if url == "/" || url == "/index.html" {
        headers.push(Header::from_bytes(&"Content-Type"[..], &"text/html; charset=utf-8"[..]).unwrap());
        let html_payload = "<meta http-equiv=\"refresh\" content=\"1;url=/\" /><h1>ACM RFC 2324 Implementation</h1><img src=\"/snap.png\">".as_bytes();
        response = Response::new(StatusCode::from(200), headers, &html_payload[..], Some(html_payload.len()), None);
      }
      else if url == "/snap.png" {
        headers.push(Header::from_bytes(&"Content-Type"[..], &"image/png"[..]).unwrap());
        match SHOT.lock() {
          Ok(v) => {
            v_clone = v.clone(); // Copy new data into this thread
            response = Response::new(StatusCode::from(200), headers, &v_clone[..], Some(v.len()), None);
          },
          _ => {
            println!("[ Warn ] Could get snap data for http response!");
            let html_payload = "DEVNULL".as_bytes(); // todo make this better
            response = Response::new(StatusCode::from(200), headers, &html_payload[..], Some(html_payload.len()), None);
          }
        }
      }
      else { // redir to "/"
        let html_payload = "<meta http-equiv=\"refresh\" content=\"0;URL='/'\" />".as_bytes();
        response = Response::new(StatusCode::from(200), headers, &html_payload[..], Some(html_payload.len()), None);
      }
      
      request.respond(response);
  }
  
}

fn webcam_thread() {
  let cap = VideoCapture::new(0);
  println!("Capturing from /dev/video0");
  
  loop {
    let image: cv::mat::Mat = cap.read().unwrap();
    
    // TODO process image
    
    // Dump into shared global var
    match SHOT.lock() {
      Ok(mut v) => {
        // Remove existing payload
        v.retain(|&_| false);
        let mut new_v_dat = image.image_encode(".png", vec![]).expect("Error encoding SHOT");
        v.append(&mut new_v_dat);
      },
      _ => {
        println!("[ Warn ] Could not dump snap!");
      }
    }
    
    println!("Snap!");
    thread::sleep(time::Duration::from_millis(SHOT_DELAY));
  }
}

fn setpot(on: bool) {
  write_to_file("/sys/class/gpio/export", "120"); // Likely throw error after writing once
  write_to_file("/sys/class/gpio/gpio120/direction", "out");
  if on {
    write_to_file("/sys/class/gpio/gpio120/value", "1");
  }
  else {
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

struct Meters {
  water_top_x: i32,
  water_top_y: i32,
  water_bot_x: i32,
  water_bot_y: i32,
  
  coffee_top_x: i32,
  coffee_top_y: i32,
  coffee_bot_x: i32,
  coffee_bot_y: i32,
  
}

fn construct_meters() -> Meters {
  Meters {
    water_top_x: 0,
    water_top_y: 0,
    water_bot_x: 0,
    water_bot_y: 0,
    
    coffee_top_x: 0,
    coffee_top_y: 0,
    coffee_bot_x: 0,
    coffee_bot_y: 0,
  }
}