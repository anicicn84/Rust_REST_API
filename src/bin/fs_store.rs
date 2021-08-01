use std::{fmt::format, iter::repeat_with};

use structopt::StructOpt;
use reqwest::blocking;
use warp::fs::file;

#[derive(StructOpt, Debug)]
// struct Arguments {
//     /// The command to run. Possible options: list-files/upload-file/delete-file
//     command: String,
//     /// The path to the file to pass in case of delete-file and upload-file
//     #[structopt(parse(from_os_str))]
//     path: std::path::PathBuf,
// }

enum Command{
    #[structopt(name="upload_file")]
    Upload {
        #[structopt(parse(from_os_str))]
        path: std::path::PathBuf,
    }, 
    #[structopt(name="delete_file")]
    Delete {
        #[structopt(parse(from_os_str))]
        path: std::path::PathBuf,
    }, 
    #[structopt(name="list_files")]
    List
}

#[derive(StructOpt, Debug)]
struct Arguments {
    #[structopt(subcommand)]
    command: Command, 

}

fn rest_upload(filename: std::path::PathBuf) {

    let url = format!("http://localhost:8080/files/{}", filename.to_str().unwrap_or("file.txt"));
    let file_bytes = std::fs::read(filename).unwrap();
    reqwest::blocking::Client::new().post(url).body(file_bytes).send().unwrap();
}

fn rest_delete(filename: std::path::PathBuf) {
    let url = format!("http://localhost:8080/files/{}", filename.to_str().unwrap_or("file.txt"));
    reqwest::blocking::Client::new().delete(url).send().unwrap();
}

fn rest_list() {
    let body = reqwest::blocking::get("http://localhost:8080/list_files").unwrap();
    let text = body.text().unwrap();
    println!("{}", text);
}

fn main() {

    let args = Arguments::from_args();

    match args.command {
        Command::Upload{path} => rest_upload(path),
        Command::Delete {path} => rest_delete(path),
        Command::List => rest_list()
    }
}
