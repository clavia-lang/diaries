use bytes::Bytes;
use http_body_util::Full;
use hyper::http::Error;
use hyper::server::conn::http1::Builder;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode, body};
use hyper_util::rt::TokioIo;
use std::sync::Arc;
use std::path::Path;
use tokio::net::TcpListener;
use markdown_html::config::Dirs;
use markdown_html::template::{render_diary, render_index};

type RequestResult = Result<Response<Full<Bytes>>, Error>;

fn handle_page(input_path: &Path, output_path: &Path) -> RequestResult {
    let output = match render_diary(input_path, output_path) {
        Ok(output) => output,
        Err(error) => {
            eprintln!("Error rendering page: {}", error);
            let builder = Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR);
            return builder.body(Full::<Bytes>::from("Error rendering page"));
        },
    };
    Ok(Response::new(Full::<Bytes>::from(output)))
}

fn handle_index(input_path: &Path, output_path: &Path, dirs: &Dirs) -> RequestResult {
    let output =  match render_index(input_path, output_path, dirs) {
        Ok(output) => output,
        Err(error) => {
            eprintln!("Error rendering index: {}", error);
            let builder = Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR);
            return builder.body(Full::<Bytes>::from("Error rendering index"));
        }
    };
    Ok(Response::new(Full::<Bytes>::from(output)))
}


async fn service(request: Request<body::Incoming>, dirs: &Dirs) -> RequestResult {
    match (request.method(), request.uri().path()) {
        (&Method::GET, path) => {
            let path = Path::new(path).strip_prefix("/").unwrap();

            // We don't known whether it is a directory or a file yet            
            let input_dir = dirs.src.join(path);
            let input_path = input_dir.with_extension("md");

            let input_dir_exists = input_dir.is_dir();
            let input_path_exists = input_path.is_file();

            if !input_path_exists && !input_dir_exists {
                let builder = Response::builder().status(StatusCode::NOT_FOUND);
                return builder.body(Full::<Bytes>::from("Page not found"));
            }

            if input_path_exists && input_dir_exists {
                let builder = Response::builder().status(StatusCode::CONFLICT);
                return builder.body(Full::<Bytes>::from("Both file and directory exists"))
            }

            if input_path_exists {
                let mut output_path = dirs.build_dir.join(path);
                output_path.set_extension("html");
                handle_page(&input_path, &output_path)
            } else if input_dir_exists {
                let output_path = dirs.build_dir.join(path).join("index.html");
                handle_index(&input_dir, &output_path, dirs)
            } else {
                unreachable!("input_path_exists and input_dir_exists are both false, which should have been handled earlier")
            }
        },
        _ => {
            let builder = Response::builder().status(StatusCode::METHOD_NOT_ALLOWED);
            builder.body(Full::<Bytes>::from("Method not allowed"))
        }
    }
}

pub async fn serve(dirs: Dirs) -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
    let listener = TcpListener::bind("127.0.0.1:2345").await?;
    let dirs = Arc::new(dirs);

    loop {
        let (stream, _) = listener.accept().await?;
        let dirs_clone = dirs.clone();

        tokio::spawn(async move {
            let io = TokioIo::new(stream);
            let http = Builder::new();
            let conn = http.serve_connection(io, service_fn(|request| service(request, &dirs_clone)));
            if let Err(error) = conn.await {
                eprintln!("failed to serve connection: {}", error);
            }
        });
    }
}
