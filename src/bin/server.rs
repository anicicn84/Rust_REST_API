//! An REST API implementation in Rust for CLI and GET/POST/DELETE file requests.
use futures::{StreamExt, TryFutureExt};
use structopt::StructOpt;
use warp::Filter;

use std::{path::PathBuf, sync::Arc};

#[derive(StructOpt)]
struct Arguments {
    /// path where the uploaded files will be stored on the server side
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

#[tokio::main]
async fn main() {
    use env_logger::Env;
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let args = Arguments::from_args();

    if !args.path.exists() {
        std::fs::create_dir_all("files").unwrap();
    }

    let path_to_working_dir = Arc::new(args.path.clone());

    //
    let get_route = warp::path("files")
        .and(warp::get())
        .and(warp::path::tail())
        .map(|path| format!("GET: got path: {:?}", path));

    let delete_route = {
        let path_to_working_dir = path_to_working_dir.clone();
        warp::path("files")
            .and(warp::delete())
            .and(warp::path::tail())
            .and_then(move |path: warp::path::Tail| {
                delete_handler(path_to_working_dir.clone(), path)
            })
    };

    let post_route = {
        let path_to_working_dir = path_to_working_dir.clone();
        warp::path("files")
        .and(warp::post())
        .and(warp::path::tail())
        .and(warp::body::bytes())
        .and_then(
            move |path: warp::path::Tail, body: warp::hyper::body::Bytes| {
                post_handler(path_to_working_dir.clone(), path, body)
            },
        )
    };

    let list_route = {
        let path_to_working_dir = path_to_working_dir.clone();
        warp::path("list_files")
        .and(warp::get())
        .and_then( move || {
            list_dir_handler(path_to_working_dir.clone())
        })
    };

    let routes = warp::path::end()
        .map(|| "what?")
        .or(get_route)
        .or(post_route)
        .or(list_route)
        .or(delete_route)
        .with(warp::log::custom(|info| {
            log::info!(
                "{:?} {:?} \n {:#?}",
                info.method(),
                info.path(),
                info.request_headers()
            );
        }));

    //
    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
}

async fn delete_handler(
    path_to_working_dir: Arc<PathBuf>,
    path: warp::path::Tail,
) -> Result<String, warp::Rejection> {
    let file_name = {
        let mut temp_path = (*path_to_working_dir).clone();
        temp_path.push(path.as_str());
        temp_path
    };
    tokio::fs::remove_file(&file_name)
        .await
        .map(|_| "File deleted!".to_string())
        .map_err(|e| {
            log::error!("error writing file: {}", e);
            warp::reject::reject()
        })
}

async fn post_handler(
    path_to_working_dir: Arc<PathBuf>,
    path: warp::path::Tail,
    body: warp::hyper::body::Bytes,
) -> Result<String, warp::Rejection> {
    let content = std::str::from_utf8(&body);
    log::info!("POST got path: {:?}, with content {:?}", path, content);

    let file_name = {
        let mut temp_path = (*path_to_working_dir).clone();
        temp_path.push(path.as_str());
        temp_path
    };

    tokio::fs::write(&file_name, &body)
        .await
        .map_err(|e| {
            log::error!("error writing file: {}", e);
            warp::reject::reject()
        })
        .map(|_| "Success".to_string())
}


async fn list_dir_handler(path_to_working_dir: Arc<PathBuf>) -> Result<String, warp::Rejection> {
    let read_dir_stream = tokio::fs::read_dir(path_to_working_dir.as_ref())
    .await
    .map_err(|e|{
        log::error!("error reading directory : {}", e);
        warp::reject::reject()
    })?;
    let dirs: Vec<_> = tokio_stream::wrappers::ReadDirStream::new(read_dir_stream).collect().await;
    let all_files = dirs.into_iter()
    .filter_map(|dir_entry| dir_entry.ok())
    .map(|ok_dir_entry| ok_dir_entry.path())
    .fold(String::new(), |mut acc, path| {
        let string_path = path.to_str().unwrap_or("");
        acc.extend(string_path.chars());
        acc.extend("\n".chars());
        acc
    });
    Ok(all_files)
}