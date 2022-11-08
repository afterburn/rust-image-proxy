mod fs;

use crate::fs::write_file;
use actix_web::{
    get,
    web::{self, Bytes},
    App, HttpRequest, HttpResponse, HttpServer,
};
use dotenv::dotenv;
use fs::mkdir;
use image::{imageops::FilterType, DynamicImage, EncodableLayout};
use serde::Deserialize;
use std::env;
use std::{
    fs::File,
    io::{BufReader, Read, Write},
};
use webp::Encoder;

extern crate dotenv;

// Load image from url and place in memory.
async fn get_image_bytes(url: String) -> anyhow::Result<Bytes> {
    let bytes = reqwest::get(url).await?.bytes().await?;
    return Ok(bytes);
}

// Convert bytes into webp format.
fn image_to_webp(img: DynamicImage, webp_path: String) -> Result<String, std::io::Error> {
    let encoder = Encoder::from_image(&img).unwrap();
    let encoded_webp = encoder.encode(65f32);

    let mut webp_image = File::create(webp_path.to_string()).unwrap();
    match webp_image.write_all(encoded_webp.as_bytes()) {
        Ok(_) => Ok(webp_path),
        Err(err) => Err(err),
    }
}

#[derive(Debug, Deserialize)]
pub struct ImageRequest {
    url: String,
    w: Option<u32>,
    h: Option<u32>,
}

#[get("/")]
async fn proxy_image(req: HttpRequest, params: web::Query<ImageRequest>) -> HttpResponse {
    // Generate id
    let hash = format!("{:?}", md5::compute(params.url.clone()));

    // Attempt to load image from disk, otherwise download and store.
    let download_path = format!("./downloads/{}", hash);
    let bytes: Bytes;
    if std::path::Path::new(&download_path).exists() {
        // Read image from disk.
        let f = File::open(download_path).unwrap();
        let mut reader = BufReader::new(f);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer).unwrap();
        bytes = Bytes::from(buffer);
    } else {
        // Download image and store to disk.
        bytes = match get_image_bytes(params.url.clone()).await {
            Ok(bytes) => bytes,
            Err(_) => todo!(),
        };
        write_file(&bytes, format!("./downloads/{}", hash)).unwrap();
    }

    // Create identifier for the webp image.
    let ws = params.w.unwrap_or(0);
    let hs = params.h.unwrap_or(0);
    let identifier = format!("{}.{}x{}", hash, ws, hs);
    let mut webp_path = format!("./webp/{}.webp", identifier);

    // Check to see if we need to resize or if its already available in cache.
    if !std::path::Path::new(&webp_path).exists() {
        // Convert bytes to DynamicImage.
        let mut img = image::load_from_memory(&bytes).unwrap();
        let original_width = img.width() as f32;
        let original_height = img.height() as f32;

        // Perform resize if it is desired.
        if params.w.is_some() || params.h.is_some() {
            let width: u32;
            let height: u32;

            if params.w.is_some() && params.h.is_none() {
                let aspect_ratio = original_height / original_width;
                width = params.w.unwrap();
                height = (width as f32 * aspect_ratio).ceil() as u32;
            } else if params.w.is_none() && params.h.is_some() {
                let aspect_ratio = original_width / original_height;
                height = params.h.unwrap();
                width = (height as f32 * aspect_ratio).ceil() as u32;
            } else {
                width = params.w.unwrap();
                height = params.h.unwrap();
            }

            img = DynamicImage::resize_exact(&img, width, height, FilterType::Gaussian);
        }

        // Convert to webp
        webp_path = image_to_webp(img, webp_path).unwrap();
    }

    // Serve webp
    let file = actix_files::NamedFile::open_async(webp_path).await.unwrap();
    file.into_response(&req)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    mkdir("./downloads".to_owned()).unwrap();
    mkdir("./webp".to_owned()).unwrap();

    let port: u16 = env::var("PORT").unwrap().parse().unwrap();
    HttpServer::new(|| App::new().service(proxy_image))
        .bind(("127.0.0.1", port))?
        .run()
        .await
}
