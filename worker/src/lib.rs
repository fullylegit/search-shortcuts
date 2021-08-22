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

type Result<T, E = JsValue> = std::result::Result<T, E>;

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
        "interest-cohort",
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
    headers.set("Permissions-Policy", &disabled_features)?;
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

fn osdf() -> Result<Response> {
    let headers = default_headers()?;
    headers.set("Content-Type", "application/opensearchdescription+xml")?;
    Response::new_with_opt_str_and_init(
        Some(include_str!("../../resources/osdf.xml")),
        ResponseInit::new().status(200).headers(&headers),
    )
}

fn index() -> Result<Response> {
    let headers = default_headers()?;
    headers.set("Content-Type", "text/html")?;
    Response::new_with_opt_str_and_init(
        Some(include_str!("../../resources/index.html")),
        ResponseInit::new().status(200).headers(&headers),
    )
}

fn redirect(query: &str) -> Result<Response> {
    let args: Args = serde_qs::from_str(query)
        .map_err(|err| format!("Failed to parse query string: {:?}", err))?;
    match args.q {
        Some(query) => {
            let headers = default_headers()?;
            let redirect_url = query_to_url(&query)
                .map_err(|err| format!("Failed to get redirect url: {:?}", err))?;
            headers.set("Location", redirect_url.as_str())?;
            return Response::new_with_opt_str_and_init(
                None,
                ResponseInit::new().status(303).headers(&headers),
            );
        }
        None => index(),
    }
}

#[wasm_bindgen]
pub async fn handle_request(request: Request) -> Result<Response> {
    utils::set_panic_hook();
    let url = Url::parse(&request.url())
        .map_err(|err| format!("Failed to parse request url: {:?}", err))?;

    match url.path() {
        "/" => match url.query() {
            Some(query) => redirect(query),
            None => index(),
        },
        "/osdf.xml" => osdf(),
        _ => {
            let headers = default_headers()?;
            Response::new_with_opt_str_and_init(
                None,
                ResponseInit::new().status(404).headers(&headers),
            )
        }
    }
}
