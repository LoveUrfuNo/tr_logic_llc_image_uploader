#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
extern crate rocket_contrib;
extern crate multipart;
#[macro_use]
extern crate log;
extern crate chrono;
extern crate env_logger;
extern crate http;
extern crate rustc_serialize;
extern crate base64;
extern crate reqwest;
extern crate image;

use rocket::{Data, Response, State, Request};
use rocket::http::{ContentType, Status};
use rocket_contrib::json::Json;

use multipart::server::Multipart;

use std::io::{Result, Read, Write, Cursor};
use std::collections::HashMap;

use chrono::Local;
use env_logger::Builder;
use log::LevelFilter;

use crate::save::{save_image, SUCCESS_MSG};
use crate::utils::{get_filename_from_url, get_response_by_status_and_errors};

mod utils;
mod save;

#[catch(404)]
fn not_found(req: &Request) -> String {
    warn!("404 with: {:?}", req);

    format!("Sorry, '{}' is not a valid path", req.uri())
}

#[catch(500)]
fn internal_error(req: &Request) -> String {
    error!("500 with: {:?}", req);

    format!("Sorry, we messed up")
}

#[post("/upload", format = "text/plain", data = "<url>")]
fn upload_by_url(dir_name: State<String>, url: String) -> Result<Response<'static>> {
    let mut response = Response::new();
    response.set_header(ContentType::Plain);

    let mut result = match reqwest::get(&url) {
        Ok(res) => res,
        Err(err) => {
            response.set_status(Status::BadRequest);
            response.set_sized_body(Cursor::new(format!("Incorrect url: {}", err.to_string())));

            return Ok(response);
        }
    };

    let filename = get_filename_from_url(url.clone());

    let mut bytes = vec![];
    result.read_to_end(&mut bytes);

    let (status, body) = save_image(
        dir_name.inner().to_owned(), filename.to_owned(), &bytes);
    response.set_status(status);
    response.set_sized_body(body);

    Ok(response)
}

#[post("/upload", format = "application/json", data = "<data>")]
fn upload_json(dir_name: State<String>, data: Json<HashMap<String, String>>) -> Result<Response<'static>> {
    let filename = data.get("filename").unwrap();

    let base64_data = data.get("data").unwrap();
    let bytes = base64::decode(base64_data).unwrap();

    let mut response = Response::new();
    response.set_header(ContentType::Plain);

    let (status, body) = save_image(
        dir_name.inner().to_owned(), filename.to_owned(), &bytes);
    response.set_status(status);
    response.set_sized_body(body);

    Ok(response)
}

#[post("/upload", format = "multipart/form-data", data = "<data>")]
fn upload(dir_name: State<String>, cont_type: &ContentType, data: Data) -> Result<Response<'static>> {
    let (_, boundary) = match cont_type.params().find(|&(key, _)| key == "boundary") {
        Some(bnd) => bnd,
        None => {
            let mut response = Response::new();
            response.set_header(ContentType::Plain);
            response.set_status(Status::BadRequest);
            response.set_sized_body(Cursor::new("Missing boundary"));

            return Ok(response);
        }
    };

    let mut statuses = Vec::new();
    let mut errors = Vec::new();

    Multipart::with_body(data.open(), boundary)
        .foreach_entry(|mut entry| {
            let filename = entry.headers.filename.unwrap();

            let mut bytes = vec![];
            entry.data.read_to_end(&mut bytes);

            let (status, body) = save_image(dir_name.inner().to_owned(), filename, &bytes);
            statuses.push(status);

            let body_str = body.into_inner();
            if body_str != SUCCESS_MSG {
                errors.push(body_str)
            }
        });

    get_response_by_status_and_errors(statuses, errors)
}

fn main() {
    Builder::new()
        .format(|buf, record| {
            writeln!(buf,
                     "{} [{}] - {}",
                     Local::now().format("%Y-%m-%d %H:%M:%S"),
                     record.level(),
                     record.args()
            )
        })
        .filter(None, LevelFilter::Info)
        .init();

    rocket::ignite()
        .mount("/", routes![upload, upload_json, upload_by_url])
        .register(catchers![not_found, internal_error])
        .manage(rocket::ignite().config().get_string("dir_name").unwrap())
        .launch();
}