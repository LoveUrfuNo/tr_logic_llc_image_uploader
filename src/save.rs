use std::io::{Result, Write, Read, Cursor};
use std::path::Path;

use rocket::http::{ContentType, Status};

use image::FilterType;

pub const FILE_EXIST_ERR_MSG: &str = "file already exists";
pub const FILE_SAVED_MSG: &str = "was saved";
pub const SUCCESS_MSG: &str = "successfully";

pub fn save_preview_err_case(filename: String, err_str: String) -> (Status, Cursor<String>) {
    let err_msg = format!("{}: Couldn't save the preview. {}", filename, err_str);
    info!("{}", err_msg);

    (Status::InternalServerError, Cursor::new(err_msg))
}

pub fn save_preview(dir_name: String, file_path: String, filename: String) -> (Status, Cursor<String>) {
    match image::open(file_path) {
        Ok(mut img) => {
            img = img.resize_exact(100, 100, FilterType::Nearest);
            match img.save(format!("{}/previews/{}", dir_name, filename)) {
                Ok(_ok) => (Status::Ok, Cursor::new(SUCCESS_MSG.to_string())),
                Err(err) => save_preview_err_case(filename, err.to_string())
            }
        },
        Err(err) => save_preview_err_case(filename, err.to_string())
    }
}

pub fn save_image(dir_name: String, filename: String, bytes: &[u8]) -> (Status, Cursor<String>) {
    let status;
    let body;

    let file_path = format!("{}/{}", dir_name, filename);

    if Path::new(&file_path.clone()).exists() && filename.capacity() > 0 {
        let err_msg = format!("{}: {}", filename, FILE_EXIST_ERR_MSG.to_string());

        status = Status::Conflict;
        body = Cursor::new(err_msg.clone());

        info!("{}", err_msg);
    } else {
        match std::fs::File::create(file_path.clone()) {
            Ok(mut file) => {
                file.write(&bytes);
                info!("{}: {}", filename, FILE_SAVED_MSG);

                let (s, b) = save_preview(dir_name, file_path.clone(), filename.clone());
                status = s;
                body = b;
            },
            Err(err) => {
                let err_code = err.raw_os_error().unwrap();
                let err_msg = if err_code == 21 {
                    status = Status::BadRequest;

                    format!("{}: {}", filename, "Wrong filename".to_string())
                } else {
                    status = Status::InternalServerError;

                    err.to_string()
                };

                info!("{}", err_msg);
                body = Cursor::new(err_msg);
            }
        }
    }

    (status, body.clone())
}



