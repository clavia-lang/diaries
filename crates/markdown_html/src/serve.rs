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
use markdown_html::convert::markdown_to_html;

async fn service(request: Request<body::Incoming>, input_dir: &Path, output_dir: &Path) -> Result<Response<Full<Bytes>>, Error> {
    match (request.method(), request.uri().path()) {
        (&Method::GET, "/") => {
            unimplemented!("list files and directories")
        }
        (&Method::GET, path) => {
            let path = Path::new(path).strip_prefix("/").unwrap();
            let mut input_path = input_dir.join(path);
            input_path.set_extension("md");

            if !input_path.exists() {
                let builder = Response::builder().status(StatusCode::NOT_FOUND);
                return builder.body(Full::<Bytes>::from("Page not found"));
            }

            let mut output_path = output_dir.join(path);
            output_path.set_extension("html");

            let output = match markdown_to_html(&input_path, &output_path) {
                Ok(output) => output,
                Err(error) => {
                    eprintln!("Error rendering Markdown: {}", error);
                    let builder = Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR);
                    return builder.body(Full::<Bytes>::from("Error rendering Markdown"));
                },
            };
            println!("{:?} {:?}", input_path, output_path);
            Ok(Response::new(Full::<Bytes>::from(output)))
        },
        _ => {
            let builder = Response::builder().status(StatusCode::METHOD_NOT_ALLOWED);
            builder.body(Full::<Bytes>::from("Method not allowed"))
        }
    }
}

pub async fn serve(input_dir: &Path, output_dir: &Path) -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
    let listener = TcpListener::bind("127.0.0.1:2345").await?;

    loop {
        let (stream, _) = listener.accept().await?;
        let input_dir_clone = Arc::new(input_dir.to_path_buf());
        let output_dir_clone = Arc::new(output_dir.to_path_buf());

        tokio::spawn(async move {
            let io = TokioIo::new(stream);
            let http = Builder::new();
            let conn = http.serve_connection(io, service_fn(|request| service(request, &input_dir_clone, &output_dir_clone)));
            if let Err(error) = conn.await {
                eprintln!("failed to serve connection: {}", error);
            }
        });
    }
}
