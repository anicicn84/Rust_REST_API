//! An REST API implementation in Rust for CLI and GET/POST/DELETE file requests.

use structopt::StructOpt;
use warp::{
    http::StatusCode,
    multipart::{FormData, Part},
    Filter, Rejection, Reply,
};

use bytes::BufMut;
use futures::TryStreamExt;
use std::convert::Infallible;
use uuid::Uuid;

#[derive(StructOpt)]
struct Cli {
    /// The command to run. Possible options: list-files/upload-file/delete-file
    command: String,
    /// The path to the file to pass in case of delete-file and upload-file
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

fn rest_upload(filename: std::path::PathBuf) {}
fn rest_delete(filename: std::path::PathBuf) {}
fn rest_list() {}

#[tokio::main]
async fn main(){
    let args = Cli::from_args();

    match args.path.exists() {
        false => panic!("File does not exist {:?}", args.path),
        _ => (),
    }

    match args.command.as_str() {
        "upload-file" => rest_upload(args.path),
        "delete-file" => rest_delete(args.path),
        "list-files" => rest_list(),
        _ => panic!("No command {} found", args.command),
    }

    let upload_route = warp::path("upload")
        .and(warp::post())
        .and(warp::multipart::form().max_length(5_000_000))
        .and_then(upload);


    //tokio::fs::read_dir -> to read the directory's files and to get them inside the list-files
    
    ///GET  /files -> serves files from the given path. File download.
    let download_route = warp::path("files").and(warp::fs::dir("./files/"));

    let router = upload_route.or(download_route).recover(handle_rejection);
    println!("Server started at localhost:8080");

    warp::serve(router)
        .run(([0, 0, 0, 0], 8080))
        .await;
}

/// upload route definition
async fn upload(form: FormData) -> Result<impl Reply, Rejection> {
    let parts: Vec<Part> = form.try_collect().await.map_err(|e| {
        eprintln!("form error: {}", e);
        warp::reject::reject()
    })?;

    for p in parts {
        if p.name() == "file" {
            let content_type = p.content_type();
            let file_ending;
            match content_type {
                Some(file_type) => match file_type {
                    "application/pdf" => {
                        file_ending = "pdf";
                    }
                    "image/png" => {
                        file_ending = "png";
                    }
                    "text/plain" => {
                        file_ending = "txt";
                    }
                    v => {
                        eprintln!("invalid file type found: {}", v);
                        return Err(warp::reject::reject());
                    }
                },
                None => {
                    eprintln!("file type could not be determined");
                    return Err(warp::reject::reject());
                }
            }

            let value = p
                .stream()
                .try_fold(Vec::new(), |mut vec, data| {
                    vec.put(data);
                    async move { Ok(vec) }
                })
                .await
                .map_err(|e| {
                    eprintln!("reading file error: {}", e);
                    warp::reject::reject()
                })?;

            let file_name = format!("./files/{}.{}", Uuid::new_v4().to_string(), file_ending);
            tokio::fs::write(&file_name, value).await.map_err(|e| {
                eprint!("error writing file: {}", e);
                warp::reject::reject()
            })?;
            println!("created file: {}", file_name);
        }
    }

    Ok("success")
}

/// handles Rejection and gives info what went wrong
async fn handle_rejection(err: Rejection) -> std::result::Result<impl Reply, Infallible> {
    let (code, message) = 
        if err.is_not_found() {
            (StatusCode::NOT_FOUND, "Not Found".to_string())
        } else if err.find::<warp::reject::PayloadTooLarge>().is_some() {
            (StatusCode::BAD_REQUEST, "Payload too large".to_string())
        } else {
            eprintln!("unhandled error: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error".to_string(),
            )
        };

    Ok(warp::reply::with_status(message, code))
}
