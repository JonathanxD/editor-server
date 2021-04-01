//     editor-server - A HTTP server to interface file edition.
//
//         The MIT License (MIT)
//
//      Copyright (c) Jonathan H. R. Lopes (https://github.com/JonathanxD) <jhrldev@gmail.com>
//      Copyright (c) contributors
//
//      Permission is hereby granted, free of charge, to any person obtaining a copy
//      of this software and associated documentation files (the "Software"), to deal
//      in the Software without restriction, including without limitation the rights
//      to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
//      copies of the Software, and to permit persons to whom the Software is
//      furnished to do so, subject to the following conditions:
//
//      The above copyright notice and this permission notice shall be included in
//      all copies or substantial portions of the Software.
//
//      THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
//      IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//      FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
//      AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//      LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
//      OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
//      THE SOFTWARE.
#![feature(stmt_expr_attributes)]
#![feature(with_options)]

use clap::{App, Arg};
use regex::Regex;
use std::env;
use std::fmt::{Display, Formatter};
use std::path::Path;
use std::fs::File;
use std::sync::{Arc, Mutex};
use warp::Filter;
use warp;
use std::io::{Read, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let only_numbers = Regex::new(r"\d+").unwrap();
    let matches = App::new("editor-server")
        .version("1.0")
        .author("Jonathan H. R. Lopes <jhrldev@gmail.com>")
        .about("An utility for interfacing file edition through HTTP for integration between different tools.")
        .arg(Arg::new("PORT")
            .validator_regex(only_numbers, "Only numbers are allowed")
            .short('p')
            .long("port")
            .about("Sets the port to open HTTP Server.")
            .takes_value(true)
            .required(false))
        .arg(Arg::new("FILE")
            .about("Sets the file to read from and write to.")
            .required(true)
            .index(1))
        .get_matches();

    let env_port = env::var("EDITOR_SERVER_PORT").map(|s| s.parse::<u16>().unwrap());
    let arg_port = matches.value_of("PORT").map(|s| s.parse::<u16>().unwrap());
    let port = arg_port.or(env_port.ok())
        .ok_or(Error("Missing port to bind. Please specify a valid port through -p parameter or $EDITOR_SERVER_PORT environment variable!".to_string()))?;

    let file_path = matches.value_of("FILE").map(|s| Path::new(s)).unwrap();
    let file = if file_path.exists() {
        File::with_options().append(true).truncate(false).write(true).read(true).open(file_path)?
    } else {
        File::create(file_path)?
    };

    let file_arc = Arc::new(Mutex::new(file));
    let content_buffer_string = Arc::new(Mutex::new(String::new()));

    {
        let file = Arc::clone(&file_arc);
        let mut locked_file = file.lock().unwrap();
        let mut buffer = content_buffer_string.lock().unwrap();

        locked_file.read_to_string(&mut buffer).unwrap();
    }

    println!("Starting editor-server on port {}...", port);

    let content_buffer_for_read = content_buffer_string.clone();
    let read = warp::get()
        .and(warp::path("read"))
        .map(move || {
            let contents = content_buffer_for_read.clone();
            let contents_data = contents.lock().unwrap();

            contents_data.clone()
        });

    let content_buffer_for_write = content_buffer_string.clone();
    let write = warp::post()
        .and(warp::path("write"))
        .and(warp::body::bytes())
        .map(move |data: warp::hyper::body::Bytes| {
            let contents = content_buffer_for_write.clone();
            let mut contents_data = contents.lock().unwrap();
            let bytes = &data[..];

            contents_data.clear();
            contents_data.push_str(String::from_utf8_lossy(bytes).as_ref());
            format!("{}", contents_data.len())
        });


    let content_buffer_for_reload = content_buffer_string.clone();
    let file_for_reload = file_arc.clone();
    let reload = warp::get()
        .and(warp::path("reload"))
        .map(move || {
            let contents = content_buffer_for_reload.clone();
            let mut contents_data = contents.lock().unwrap();

            let file = file_for_reload.clone();
            let mut file_data = file.lock().unwrap();

            file_data.read_to_string(&mut contents_data).unwrap();

            format!("{}", contents_data.len())
        });

    let content_buffer_for_flush = content_buffer_string.clone();
    let file_for_flush = file_arc.clone();
    let flush = warp::get()
        .and(warp::path("save"))
        .map(move || {
            let contents = content_buffer_for_flush.clone();
            let contents_data = contents.lock().unwrap();

            let file = file_for_flush.clone();
            let mut file_data = file.lock().unwrap();

            let content_bytes = contents_data.as_bytes();
            file_data.set_len(0).unwrap();
            file_data.write_all(content_bytes).unwrap();
            file_data.sync_data().unwrap();
            file_data.sync_all().unwrap();

            format!("{}", contents_data.len())
        });

    let content_buffer_for_close = content_buffer_string.clone();
    let file_for_close = file_arc.clone();
    let close = warp::get()
        .and(warp::path("close"))
        .map(move || {
            let contents = content_buffer_for_close.clone();
            let contents_data = contents.lock().unwrap();

            let file = file_for_close.clone();
            let mut file_data = file.lock().unwrap();

            let content_bytes = contents_data.as_bytes();
            file_data.set_len(0).unwrap();
            file_data.write_all(content_bytes).unwrap();
            file_data.sync_data().unwrap();
            file_data.sync_all().unwrap();

            std::process::exit(0);
            #[allow(unreachable_code)]
            "Ok"
        });

    let or = read.or(write).or(flush).or(reload).or(close);

    let serve = warp::serve(or)
        .run(([0, 0, 0, 0], port))
        .await;

    Ok(serve)
}

#[derive(Debug)]
struct Error(String);

impl std::error::Error for Error {
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}