mod utils;

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

#[derive(Debug, Deserialize)]
struct Args {
    q: Option<String>,
}

fn osdf() -> Response {
    let headers = Headers::new().unwrap();
    headers.set("Content-Type", "text/xml").unwrap();
    Response::new_with_opt_str_and_init(
        Some(include_str!("../../resources/osdf.xml")),
        ResponseInit::new().status(200).headers(&headers),
    )
    .unwrap()
}

fn index() -> Response {
    let headers = Headers::new().unwrap();
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
            let headers = Headers::new().unwrap();
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
        _ => Response::new_with_opt_str_and_init(None, ResponseInit::new().status(404)).unwrap(),
    }
}
