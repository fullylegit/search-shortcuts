mod utils;

use itertools::Itertools;
use search_shortcuts::query_to_url;
use serde::Deserialize;
use url::Url;
use wasm_bindgen::prelude::*;
use web_sys::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

fn default_headers() -> Result<Headers, JsValue> {
    let features = [
        "accelerometer",
        "ambient-light-sensor",
        "autoplay",
        "battery",
        "camera",
        "display-capture",
        "document-domain",
        "encrypted-media",
        "execution-while-not-rendered",
        "execution-while-out-of-viewport",
        "fullscreen",
        "geolocation",
        "gyroscope",
        "layout-animations",
        "legacy-image-formats",
        "magnetometer",
        "microphone",
        "midi",
        "navigation-override",
        "oversized-images",
        "payment",
        "picture-in-picture",
        "publickey-credentials-get",
        "sync-xhr",
        "usb",
        "vr",
        "wake-lock",
        "screen-wake-lock",
        "web-share",
        "xr-spatial-tracking",
    ];
    let disabled_features = features
        .iter()
        .map(|feature| format!("{} 'none'", feature))
        .join("; ");

    let headers = Headers::new()?;
    headers.set("Referrer-Policy", "no-referrer")?;
    headers.set("X-XSS-Protection", "1; mode=block")?;
    headers.set("X-Frame-Options", "DENY")?;
    headers.set("X-Content-Type-Options", "nosniff")?;
    headers.set("Feature-Policy", &disabled_features)?;
    headers.set("Content-Security-Policy", "default-src 'self'")?;
    headers.set(
        "Strict-Transport-Security",
        "max-age=63072000; includeSubDomains; preload ",
    )?;
    headers.set("Cross-Origin-Embedder-Policy", "require-corp")?;
    headers.set("Cross-Origin-Resource-Policy", "cross-origin")?;
    headers.set("Cross-Origin-Opener-Policy", "same-origin")?;
    Ok(headers)
}

#[derive(Debug, Deserialize)]
struct Args {
    q: Option<String>,
}

fn osdf() -> Response {
    let headers = default_headers().unwrap();
    headers
        .set("Content-Type", "application/opensearchdescription+xml")
        .unwrap();
    Response::new_with_opt_str_and_init(
        Some(include_str!("../../resources/osdf.xml")),
        ResponseInit::new().status(200).headers(&headers),
    )
    .unwrap()
}

fn index() -> Response {
    let headers = default_headers().unwrap();
    headers.set("Content-Type", "text/html").unwrap();
    Response::new_with_opt_str_and_init(
        Some(include_str!("../../resources/index.html")),
        ResponseInit::new().status(200).headers(&headers),
    )
    .unwrap()
}

fn redirect(query: &str) -> Response {
    let args: Args = serde_qs::from_str(query).unwrap();
    match args.q {
        Some(query) => {
            let headers = default_headers().unwrap();
            let redirect_url = query_to_url(&query).unwrap();
            headers.set("Location", redirect_url.as_str()).unwrap();
            return Response::new_with_opt_str_and_init(
                None,
                ResponseInit::new().status(303).headers(&headers),
            )
            .unwrap();
        }
        None => index(),
    }
}

#[wasm_bindgen]
pub async fn handle_request(request: Request) -> Response {
    utils::set_panic_hook();
    let url = Url::parse(&request.url()).unwrap();

    match url.path() {
        "/" => match url.query() {
            Some(query) => redirect(query),
            None => index(),
        },
        "/osdf.xml" => osdf(),
        _ => Response::new_with_opt_str_and_init(
            None,
            ResponseInit::new()
                .status(404)
                .headers(&default_headers().unwrap()),
        )
        .unwrap(),
    }
}
