use actix_web::{get, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use include_dir::{include_dir, Dir};
use std::io;

static ASSETS: Dir = include_dir!("$CARGO_MANIFEST_DIR/dist");

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().content_type("text/html").body(
        ASSETS
            .get_file("index.html")
            .expect("no index.html in dist/")
            .contents(),
    )
}

async fn assets(req: HttpRequest) -> impl Responder {
    let path = &req.path()[1..];
    let mime = if let Some(pos) = path.rfind('.') {
        match &path[pos + 1..] {
            "html" => "text/html",
            "js" => "application/javascript",
            "css" => "text/css",
            "wasm" => "application/wasm",
            "txt" => "text/plain",
            _ => "application/octet-stream",
        }
    } else {
        "application/octet-stream"
    };
    Ok::<HttpResponse, io::Error>(
        HttpResponse::Ok().content_type(mime).body(
            ASSETS
                .get_file(path)
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "not found"))?
                .contents(),
        ),
    )
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    let server = HttpServer::new(|| {
        App::new()
            .service(index)
            .route("/{f:.*}", web::get().to(assets))
    })
    .workers(1);
    println!("Listening on http://localhost:8080");
    server.bind(("localhost", 8080))?.run().await
}
