use rocket::http::{ContentType, Status};
use rocket::Response;

use std::io::{Result, Cursor};

use crate::save::SUCCESS_MSG;

pub fn get_filename_from_url(url: String) -> String {
    let url_parts: Vec<&str> = url.split('/').collect();
    let filename_probably_with_url_params = url_parts.last().unwrap();
    let filename_parts: Vec<&str> = filename_probably_with_url_params.split('?').collect();

    if filename_parts.capacity() > 1 {
        filename_parts.first().unwrap().to_string()
    } else {
        filename_probably_with_url_params.to_string()
    }
}

pub fn get_response_by_status_and_errors(statuses: Vec<Status>, errors: Vec<String>) -> Result<Response<'static>> {
    let mut response = Response::new();
    response.set_header(ContentType::Plain);

    for status in statuses {
        if Status::Ok != status {
            response.set_status(status);
            response.set_sized_body(Cursor::new(errors.join("; ")));

            return Ok(response);
        }
    }

    response.set_status(Status::Ok);
    response.set_sized_body(Cursor::new(SUCCESS_MSG));

    Ok(response)
}