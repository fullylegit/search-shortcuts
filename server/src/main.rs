mod errors;
use errors::{Error, Result};

use actix_web::middleware::{Compress, DefaultHeaders, Logger};
use actix_web::web::Query;
use actix_web::{get, App, HttpResponse, HttpServer};
use itertools::Itertools;
use openssl::ssl::{SslAcceptor, SslAcceptorBuilder, SslFiletype, SslMethod};
use search_shortcuts::query_to_url;
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize)]
struct Args {
    q: Option<String>,
}

#[get("/")]
async fn index(args: Query<Args>) -> Result<HttpResponse> {
    Ok(match &args.q {
        Some(query) => {
            let redirect_url = query_to_url(&query)?;
            HttpResponse::SeeOther()
                .header("Location", redirect_url.as_str())
                .finish()
        }
        None => HttpResponse::Ok()
            .content_type("text/html")
            .body(include_str!("../../resources/index.html")),
    })
}

#[get("/osdf.xml")]
async fn osdf() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("application/opensearchdescription+xml")
        .body(include_str!("../../resources/osdf.xml"))
}

fn setup_ssl() -> Result<SslAcceptorBuilder> {
    let key_filename = match env::var("TLS_KEY_FILE") {
        Ok(key_filename) => key_filename,
        Err(env::VarError::NotPresent) => return Err(Error::EnvVarMissing("TLS_KEY_FILE")),
        Err(env::VarError::NotUnicode(_)) => return Err(Error::EnvVarInvalidUtf8("TLS_KEY_FILE")),
    };
    let cert_filename = match env::var("TLS_CERT_FILE") {
        Ok(key_filename) => key_filename,
        Err(env::VarError::NotPresent) => return Err(Error::EnvVarMissing("TLS_CERT_FILE")),
        Err(env::VarError::NotUnicode(_)) => return Err(Error::EnvVarInvalidUtf8("TLS_CERT_FILE")),
    };

    let mut builder = match SslAcceptor::mozilla_modern(SslMethod::tls()) {
        Ok(builder) => builder,
        Err(_) => return Err(Error::Tls("Couldn't initiate tls")),
    };

    if builder
        .set_private_key_file(key_filename, SslFiletype::PEM)
        .is_err()
    {
        return Err(Error::Tls("Couldn't load private key"));
    }

    if builder.set_certificate_chain_file(cert_filename).is_err() {
        return Err(Error::Tls("Couldn't load public key"));
    }

    Ok(builder)
}

fn default_headers() -> DefaultHeaders {
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

    DefaultHeaders::new()
        .header("Referrer-Policy", "no-referrer")
        .header("X-XSS-Protection", "1; mode=block")
        .header("X-Frame-Options", "DENY")
        .header("X-Content-Type-Options", "nosniff")
        .header("Feature-Policy", disabled_features)
        .header("Content-Security-Policy", "default-src 'self'")
        .header("Cross-Origin-Embedder-Policy", "require-corp")
        .header("Cross-Origin-Resource-Policy", "cross-origin")
        .header("Cross-Origin-Opener-Policy", "same-origin")
        .header(
            "Strict-Transport-Security",
            "max-age=63072000; includeSubDomains; preload ",
        )
}

fn init_logging() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
    env_logger::init();
}

#[actix_web::main]
async fn main() -> Result<()> {
    init_logging();

    let bind_addr = match env::var("BIND_ADDR") {
        Ok(bind_addr) => bind_addr,
        Err(env::VarError::NotPresent) => return Err(Error::EnvVarMissing("BIND_ADDR")),
        Err(env::VarError::NotUnicode(_)) => return Err(Error::EnvVarInvalidUtf8("BIND_ADDR")),
    };

    let ssl_builder = setup_ssl()?;

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::new(r#"%s %b "%{User-Agent}i" %T"#))
            .wrap(Compress::default())
            .wrap(default_headers())
            .service(index)
            .service(osdf)
    })
    .bind_openssl(bind_addr, ssl_builder)?
    .run()
    .await?;

    Ok(())
}
