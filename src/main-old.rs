use anyhow::Result;
use axum::{
    body::{self, Bytes},
    extract::Query,
    http::{HeaderValue, Response, StatusCode},
    response::{Html, IntoResponse},
    routing::get,
    Json, Router,
};
use http_body::{Empty, Full};
use image::EncodableLayout;
use include_dir::{include_dir, Dir};
use reqwest::header;
use serde::{de, Deserialize, Deserializer};
use serde_json::{json, Value};
use std::{fmt, fs::File, io::Write, str::FromStr};
use urlencoding::decode;
use webp::{Encoder, WebPMemory};
static TMP_DIR: Dir<'_> = include_dir!("./tmp");

async fn hello_world() -> &'static str {
    "Hello, world!"
}

async fn json() -> (StatusCode, Json<Value>) {
    (StatusCode::BAD_REQUEST, Json(json!({ "data": 42 })))
}

async fn html() -> (StatusCode, Html<&'static str>) {
    (StatusCode::OK, Html("<h1>Hello, World!</h1>"))
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ImageRequestParams {
    #[serde(default, deserialize_with = "empty_string_as_none")]
    url: Option<String>,
}

fn empty_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    T::Err: fmt::Display,
{
    let opt = Option::<String>::deserialize(de)?;
    match opt.as_deref() {
        None | Some("") => Ok(None),
        Some(s) => FromStr::from_str(s).map_err(de::Error::custom).map(Some),
    }
}

// Load image from url and place in memory.
async fn get_image_bytes(url: String) -> Result<Bytes> {
    let bytes = reqwest::get(url).await?.bytes().await?;

    return Ok(bytes);
}

fn create_dir(dir_path: String) -> Result<()> {
    std::fs::create_dir_all(dir_path)?;
    Ok(())
}

fn convert_bytes_into_webp(id: String, bytes: Bytes) -> Result<String, std::io::Error> {
    let img = image::load_from_memory(&bytes).unwrap();
    let encoder: Encoder = Encoder::from_image(&img).unwrap();
    let encoded_webp: WebPMemory = encoder.encode(65f32);

    let webp_image_path = format!("./tmp/{}.webp", id);
    let mut webp_image = File::create(webp_image_path.to_string()).unwrap();
    match webp_image.write_all(encoded_webp.as_bytes()) {
        Ok(_) => Ok(webp_image_path),
        Err(err) => Err(err),
    }
}

async fn download_and_convert_image(id: String, image_url: String) -> Result<String> {
    let image_url_decoded = match decode(&image_url) {
        Ok(it) => it.to_string(),
        Err(_) => {
            todo!()
        }
    };

    let bytes = get_image_bytes(image_url_decoded).await?;
    Ok(convert_bytes_into_webp(id, bytes)?)
}

async fn proxy_handler(params: Query<ImageRequestParams>) -> impl IntoResponse {
    let params: ImageRequestParams = params.0;

    let image_url = match params.url {
        Some(url) => url,
        None => "".to_owned(),
    };

    if image_url.is_empty() {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Bad request" })),
        );
    }

    let id = "test".to_owned();

    let webp_path = match download_and_convert_image(id, image_url).await {
        Err(_) => "".to_owned(),
        Ok(webp_path) => webp_path,
    };

    // if webp_path.is_empty() {
    //     return Err((
    //         StatusCode::INTERNAL_SERVER_ERROR,
    //         Json(json!({ "error": "Internal server error" })),
    //     ));
    // };

    match TMP_DIR.get_file("test.webp") {
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(body::boxed(Empty::new()))
            .unwrap(),
        Some(file) => Response::builder()
            .status(StatusCode::OK)
            .header(
                header::CONTENT_TYPE,
                HeaderValue::from_str("webp".as_ref()).unwrap(),
            )
            .body(body::boxed(Full::from(file.contents())))
            .unwrap(),
    }
}

#[tokio::main]
async fn main() {
    match create_dir("./tmp".to_owned()) {
        Ok(_) => {}
        Err(_) => panic!("Could not create tmp directory."),
    }

    // build our application with a single route
    let app = Router::new()
        .route("/", get(html))
        .route("/proxy", get(proxy_handler));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
