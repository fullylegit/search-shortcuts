mod utils;

use itertools::Itertools;
use search_shortcuts::query_to_url;
use serde::Deserialize;
use worker::*;

#[derive(Debug, Deserialize)]
struct Args {
    q: Option<String>,
}

fn default_headers(content_type: Option<&str>) -> Result<Headers> {
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

    let mut headers = Headers::new();
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

    if let Some(content_type) = content_type {
        headers.set("Content-Type", content_type)?;
    }

    Ok(headers)
}

fn redirect(query: &str) -> Result<Response> {
    let args: Args = serde_qs::from_str(query)
        .map_err(|err| format!("Failed to parse query string: {:?}", err))?;
    match args.q {
        Some(query) => {
            let headers = {
                let mut headers = default_headers(None)?;
                let redirect_url = query_to_url(&query)
                    .map_err(|err| format!("Failed to get redirect url: {:?}", err))?;
                headers.set("Location", redirect_url.as_str())?;
                headers
            };
            Ok(Response::empty()?.with_status(303).with_headers(headers))
        }
        None => index_page(),
    }
}

fn index_page() -> Result<Response> {
    let headers = default_headers(Some("text/html"))?;
    Ok(Response::from_html(include_str!("../../resources/index.html"))?.with_headers(headers))
}

fn osdf(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    let headers = default_headers(Some("application/opensearchdescription+xml"))?;
    Ok(Response::from_html(include_str!("../../resources/osdf.xml"))?.with_headers(headers))
}

fn index(req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    match req.url()?.query() {
        Some(query) => redirect(query),
        None => index_page(),
    }
}

#[event(fetch)]
pub async fn main(req: Request, env: Env) -> Result<Response> {
    // Optionally, get more helpful error messages written to the console in the case of a panic.
    utils::set_panic_hook();

    // Optionally, use the Router to handle matching endpoints, use ":name" placeholders, or "*name"
    // catch-alls to match on specific patterns. The Router takes some data with its `new` method
    // that can be shared throughout all routes. If you don't need any shared data, use `()`.
    let router = Router::new(());

    // Add as many routes as your Worker needs! Each route will get a `Request` for handling HTTP
    // functionality and a `RouteContext` which you can use to  and get route parameters and
    // Enviornment bindings like KV Stores, Durable Objects, Secrets, and Variables.
    router
        .get("/", index)
        .get("/osdf.xml", osdf)
        .run(req, env)
        .await
}
