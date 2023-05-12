use actix_web::http::StatusCode;
use actix_web::{get, web, App, HttpResponse, HttpServer};
use futures::Stream;
use image::{DynamicImage, ImageFormat, ImageOutputFormat};
use std::error::Error;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::task::Poll;
mod query;
use query::ImageServerQuery;

use crate::query::make_redirect_test_server_url;


// A dummy last modified timestamp to simulate 304 errors
const THRESHOLD_304: i64 = 0;

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    let args = std::env::args().collect::<Vec<String>>();
    let port = args[1].parse::<u16>().unwrap();
    std::env::set_var("PORT", port.to_string());
    HttpServer::new(|| App::new().service(get_image))
        .bind(("127.0.0.1", port))?
        .run()
        .await
}

#[get("api/image")]
async fn get_image(query: web::Query<ImageServerQuery>) -> Result<HttpResponse, actix_web::Error> {
    println!("Received request with query {:#?}", query);
    let width = query.width.unwrap_or(100);
    let height = query.height.unwrap_or(100);
    let mime = query.mime.clone().unwrap_or("PNG".to_string());
    let status = query.status.clone().unwrap_or(200);
    if let Some(last_modified) = query.last_modified {
        if last_modified > THRESHOLD_304 {
            return Ok(HttpResponse::NotModified().finish())
        }
    }
    if let Some(redirect) = query.redirect {
        if redirect > 0 {
            let mut response = HttpResponse::PermanentRedirect();
            response.append_header(("Location", make_redirect_test_server_url(&query.0)));
            return Ok(response.finish())
        }
    }
    let mut temp_file = tempfile::tempfile().unwrap();
    let img = DynamicImage::new_rgb8(width, height);
    let mime_type = match mime.to_ascii_uppercase().as_str() {
        "PNG" => image::ImageFormat::Png,
        "JPEG" | "JPG" => image::ImageFormat::Jpeg,
        "GIF" => image::ImageFormat::Gif,
        "ICO" => image::ImageFormat::Ico,
        _ => {
            return Ok(HttpResponse::BadRequest().body("Not implemented for the mime type"));
        }
    };
    img.write_to(&mut temp_file, ImageOutputFormat::from(mime_type)).unwrap();
    temp_file.seek(SeekFrom::Start(0))?;
    let stream = FileStream::new(temp_file);
    let extension = ImageFormat::from(mime_type).extensions_str()[0];
    let content_type = format!("image/{}", extension);
    let response = construct_response(status, stream, content_type);
    Ok(response)
}

fn construct_response(
    status: u16,
    stream: FileStream,
    content_type: String
) -> HttpResponse {
    match status {
        200 => HttpResponse::Ok().content_type(content_type).streaming(stream),
        300..=399 => HttpResponse::BadRequest().body("300 Status Code is NOT supported"),
        400..=499 => HttpResponse::new(StatusCode::from_u16(status).expect("status code is 400-499")),
        500..=599 => HttpResponse::new(StatusCode::from_u16(status).expect("status code is 500-599")),
        _ => HttpResponse::BadRequest().body("Cannot recognize the status code")
    }
}

pub struct FileStream {
    file: File,
    buffer: [u8; 4096],
}

impl Stream for FileStream {
    type Item = Result<bytes::Bytes, Box<dyn Error>>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let mut buffer = self.buffer;
        match self.file.read(&mut buffer) {
            Ok(len) => {
                if len == 0 {
                    Poll::Ready(None)
                } else {
                    Poll::Ready(Some(Ok(bytes::Bytes::copy_from_slice(&buffer[0..len]))))
                }
            }
            Err(err) => Poll::Ready(Some(Err(Box::new(err)))),
        }
    }
}

impl FileStream {
    pub fn new(file: File) -> Self {
        let buffer = [0; 4096];
        Self { file, buffer }
    }
}

#[cfg(test)]
mod test {
    use crate::get_image;
    use actix_web::App;
    use std::io::Write;

    /* To Run: cargo test --package image-server --bin image-server -- test::test_basic_image --exact --nocapture   */
    #[actix_rt::test]
    async fn test_basic_image() {
        let srv = actix_test::start(|| App::new().service(get_image));
        let req = srv.get("/api/image?width=500&height=500&mime=png");
        let mut res = req.send().await.unwrap();
        let body = res.body().await.unwrap();
        let mut file = std::fs::File::create("image.png").unwrap();
        file.write_all(&body).unwrap();
        file.flush().unwrap();
        let dimension = image::image_dimensions("image.png").unwrap();
        let mime_type = image::guess_format(&body).unwrap();
        assert_eq!(dimension, (500, 500));
        assert_eq!(mime_type, image::ImageFormat::Png);
        std::fs::remove_file("image.png").unwrap();
    }

    /* To Run: cargo test --package image-server --bin image-server -- test::test_304 --exact --nocapture   */
    #[actix_rt::test]
    async fn test_304() {
        let srv = actix_test::start(|| App::new().service(get_image));
        let req = srv.get("/api/image?last_modified=1");
        let res = req.send().await.unwrap();        
        assert_eq!(res.status().as_u16(), 304);
    }

    /* To Run: cargo test --package image-server --bin image-server -- test::test_redirect --exact --nocapture   */
    #[actix_rt::test]
    async fn test_redirect() {
        let srv = actix_test::start(|| App::new().service(get_image));
        let port = srv.addr().port();
        std::env::set_var("PORT", port.to_string());
        let req = srv.get("/api/image?redirect=1");
        let res = req.send().await.unwrap();    
        assert_eq!(res.status().as_u16(), 200);
    }
}
