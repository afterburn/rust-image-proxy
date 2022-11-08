use actix_web::{
    get,
    web::{self, Bytes},
    App, HttpRequest, HttpResponse, HttpServer,
};
use image::{imageops::FilterType, DynamicImage, EncodableLayout};
use serde::Deserialize;
use std::{fs::File, io::Write};
use webp::Encoder;

#[derive(Debug, Deserialize)]
pub struct ImageRequest {
    url: String,
    w: Option<u32>,
    h: Option<u32>,
}

#[get("/")]
async fn proxy_image(req: HttpRequest, params: web::Query<ImageRequest>) -> HttpResponse {
    // Generate id
    let id = "abc123".to_owned();

    // Download image
    let bytes = match get_image_bytes(params.url.clone()).await {
        Ok(v) => v,
        Err(_) => todo!(),
    };

    // Load image into memory.
    let mut img = image::load_from_memory(&bytes).unwrap();

    // Resize
    if params.w.is_some() || params.h.is_some() {
        let original_width = img.width() as f32;
        let original_height = img.height() as f32;

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
    let webp_path = image_to_webp(img, format!("./tmp/{}.webp", id)).unwrap();

    // Serve webp
    let file = actix_files::NamedFile::open_async(webp_path).await.unwrap();
    file.into_response(&req)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    match create_dir("./tmp".to_owned()) {
        Ok(_) => {}
        Err(_) => panic!("Could not create tmp directory."),
    };

    HttpServer::new(|| App::new().service(proxy_image))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}

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

fn create_dir(dir_path: String) -> anyhow::Result<()> {
    std::fs::create_dir_all(dir_path)?;
    Ok(())
}
