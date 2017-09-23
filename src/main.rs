extern crate multipart;
extern crate iron;

use std::fs;
use multipart::server::{Multipart, Entries, SaveResult};
use iron::prelude::*;
use iron::status;
use iron::modifiers::Header;
use iron::headers;
use std::env;

fn main() {
    let mut output_path = env::args().nth(1).unwrap_or("./".to_string());
    if output_path.chars().rev().nth(0).unwrap() != '/' {
        output_path += "/";
    }
    Iron::new(move |request: &mut Request| {
        handle_request(&output_path, request)
    }).http("0.0.0.0:3002")
        .expect("Failed to start server.");
}

fn handle_request(output_path: &str, request: &mut Request) -> IronResult<Response> {
    let _ = fs::create_dir_all(output_path);
    match Multipart::from_request(request) {
        Ok(mut multipart) => {
            match multipart.save().temp() {
                SaveResult::Full(entries) => save_file(output_path.to_string(), entries),
                SaveResult::Partial(_, reason) => response(Err(format!(
                    "Failed to read request: {}",
                    reason.unwrap_err()
                ))),
                SaveResult::Error(error) => response(
                    Err(format!("Failed to reading request: {}", error)),
                ),
            }
        }
        Err(_) => response(Err("The request is not multipart.".to_string())),
    }
}

fn save_file(output_path: String, entries: Entries) -> IronResult<Response> {
    for (_, files) in entries.files {
        if files.len() > 0 {
            let temp_file = &files[0];
            let file_name = temp_file
                .path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap()
                .to_string() + "." +
                &temp_file.content_type.1.to_lowercase();
            println!("Saving file: {}", file_name);
            let file_path = output_path + &file_name;
            let _ = fs::copy(&temp_file.path, file_path);
            return Ok(Response::with((
                status::Ok,
                Header(headers::ContentType::json()),
                format!("{{\"file_path\": \"{}\"}}", file_name),
            )));
        }
    }

    response(Err("Failed to find any file in the request.".to_string()))

}

fn response(result: Result<String, String>) -> IronResult<Response> {
    match result {
        Ok(file_name) => Ok(Response::with((
            status::Ok,
            Header(headers::ContentType::json()),
            format!("{{\"file_path\": \"{}\"}}", file_name),
        ))),
        Err(reason) => Ok(Response::with((
            status::BadRequest,
            Header(headers::ContentType::json()),
            format!("{{\"error\": \"{}\"}}", reason),
        ))),
    }
}
