extern crate tiny_http;
extern crate rscam;
extern crate image;

use tiny_http::*;
use rscam::{Camera, Config};

use std::{thread, time, fs};
use std::io::Write;
use std::path::Path;


fn main() {
  let webcam_handle    = thread::spawn(|| { webcam_thread();    });
  let webserver_handle = thread::spawn(|| { webserver_thread(); });
  webcam_handle.join().unwrap();
  webserver_handle.join().unwrap();
}

fn webserver_thread() {
  let listen_addr = "0.0.0.0:8080";
  let server = Server::http(listen_addr).unwrap();
  println!("Listening on {}", listen_addr);
  
  for request in server.incoming_requests() {
      println!("received request! method: {:?}, url: {:?}, headers: {:?}",
          request.method(),
          request.url(),
          request.headers()
      );
      
      let response = Response::from_string("hello world");
      request.respond(response);
  }
  
}

fn webcam_thread() {
  let cam = "/dev/video0";
  let mut camera = Camera::new(cam).unwrap();
  println!("Capturing {}", cam);
  
  let img_width = 320;
  let img_height = 240;
  
  camera.start(&Config {
    interval: (1, 10),      // 10 fps.
    //resolution: (640, 480),
    resolution: (img_width, img_height),
    //format: b"MJPG",
    format: b"YUYV",
    //format: b"YVYU",
    ..Default::default()
  }).unwrap();
  
  loop {
    let frame = camera.capture().unwrap();
    let (pic_width, pic_height) = frame.resolution;
    let colorspace = 0;
    let rowstride = 3 * pic_width;
    let vec = Vec::from(&frame[..]);
    
    // See https://stackoverflow.com/questions/5452965/fastest-approximative-methods-to-convert-yuv-to-rgba
    // See https://en.wikipedia.org/wiki/YUV#Converting_between_Y%E2%80%B2UV_and_RGB
    
    /*let pixbuf = Pixbuf::new_from_vec(vec,
                                      colorspace,
                                      false,
                                      8,
                                      pic_width as i32,
                                      pic_height as i32,
                                      rowstride as i32);
    */
    
    /*for col in 0..320 {
      for row in 0..240 {
        print!("{}, ", vec[(320 * col) + row]);
      }
      println!("");
    }*/
    
    //let mut file = fs::File::create(&format!("frame.png")).unwrap();
    //file.write_all(&frame[..]).unwrap();
    
    image::save_buffer(&Path::new("frame.png"), &vec[..], img_width, img_height, image::RGBA(8));
    println!("Snap! (vec.len() = {})", vec.len());
    thread::sleep(time::Duration::from_millis(1200));
  }
  
}