pub mod errors;
use errors::Result;

use url::Url;

fn handle_static_redirects(query: &str) -> Result<Option<Url>> {
    Ok(match query {
        "twir" => Url::parse("https://this-week-in-rust.org")?.into(),
        "abc" => Url::parse("https://www.abc.net.au/news")?.into(),
        "had" => Url::parse("https://hackaday.com/blog/")?.into(),
        "sd" => Url::parse("https://slashdot.org")?.into(),
        "/." => Url::parse("https://slashdot.org")?.into(),
        "sth" => Url::parse("https://www.servethehome.com")?.into(),
        "x" => Url::parse("https://xkcd.com")?.into(),
        "weather" => Url::parse("https://weather.bom.gov.au/location/r3dp390-canberra")?.into(),
        "gh" => Url::parse("https://github.com")?.into(),
        "bfio" => Url::parse("https://bushfire.io")?.into(),
        _ => None,
    })
}

fn handle_docs(query: &str) -> Result<Url> {
    Ok(if query == "std" {
        Url::parse("https://doc.rust-lang.org/stable/std/")?
    } else {
        let url = Url::parse("https://docs.rs/")?;
        if let Some((crate_, version)) = {
            let sep_idx = if let Some(sep_idx) = query.find('/') {
                Some(sep_idx)
            } else if let Some(sep_idx) = query.find('@') {
                Some(sep_idx)
            } else {
                None
            };
            if let Some(sep_idx) = sep_idx {
                let (crate_, version) = query.split_at(sep_idx);
                if version.len() > 1 {
                    Some((crate_, &version[1..]))
                } else {
                    None
                }
            } else {
                None
            }
        } {
            // crate + version
            url.join(&format!("{}/{}", crate_, version))?
        } else {
            // crate
            if query.ends_with('/') {
                url.join(query)?
            } else {
                url.join(&format!("{}/", query))?
            }
        }
    })
}

fn handle_github(query: &str) -> Result<Url> {
    let url = Url::parse("https://github.com/")?;
    Ok(
        if let Some(user) = {
            if let Some(query) = query.strip_prefix('@') {
                Some(query)
            } else if let Some(query) = query.strip_prefix("u/") {
                Some(query)
            } else {
                None
            }
        } {
            url.join(user)?
        } else if query.contains('/') {
            if let Some((repo, issue)) = query.split_once(' ') {
                // github treats issues and prs the same, but this distinction
                // prevents an unneccesary redirect
                if let Some(issue) = issue.strip_prefix('#') {
                    url.join(&format!("{}/issues/{}", repo, issue))?
                } else if let Some(pr) = issue.strip_prefix("!") {
                    url.join(&format!("{}/pull/{}", repo, pr))?
                } else {
                    url.join(query)?
                }
            } else {
                url.join(query)?
            }
        } else {
            let mut url = url.join("search")?;
            url.query_pairs_mut().append_pair("q", query);
            url
        },
    )
}

fn handle_wikipedia(query: &str) -> Result<Url> {
    Ok(Url::parse_with_params(
        "https://en.wikipedia.org/wiki/Special:Search",
        &[("search", query)],
    )?)
}

fn handle_stackoverflow(query: &str) -> Result<Url> {
    Ok(Url::parse_with_params(
        "https://stackoverflow.com/search",
        &[("q", query)],
    )?)
}

fn handle_docker_hub(query: &str) -> Result<Url> {
    let url = Url::parse("https://hub.docker.com/")?;
    Ok(if let Some(query) = query.strip_prefix("r/") {
        if !query.contains('/') {
            url.join("_/")?.join(query)?
        } else {
            url.join("r/")?.join(query)?
        }
    } else if let Some(query) = query.strip_prefix('/') {
        url.join("_/")?.join(query)?
    } else if let Some(query) = query.strip_prefix("_/") {
        url.join("_/")?.join(query)?
    } else if query.contains('/') {
        url.join("r/")?.join(query)?
    } else {
        let mut url = url.join("search")?;
        url.query_pairs_mut().append_pair("q", query);
        url
    })
}

fn handle_crates(query: &str) -> Result<Url> {
    Ok(Url::parse_with_params(
        "https://crates.io/search",
        &[("q", query)],
    )?)
}

fn handle_auspost(query: &str) -> Result<Url> {
    Ok(Url::parse(&format!(
        "https://auspost.com.au/mypost/track/#/details/{}",
        query
    ))?)
}

pub fn query_to_url(query: &str) -> Result<Url> {
    if let Some(url) = handle_static_redirects(query)? {
        return Ok(url);
    }
    if let Some(query) = query.strip_prefix("docs ") {
        return Ok(handle_docs(query)?);
    }
    if let Some(query) = query.strip_prefix("docs/") {
        return Ok(handle_docs(query)?);
    }
    if let Some(query) = query.strip_prefix("gh ") {
        return Ok(handle_github(query)?);
    }
    if let Some(query) = query.strip_prefix("w ") {
        return Ok(handle_wikipedia(query)?);
    }
    if let Some(query) = query.strip_prefix("so ") {
        return Ok(handle_stackoverflow(query)?);
    }
    if let Some(query) = query.strip_prefix("dh ") {
        return Ok(handle_docker_hub(query)?);
    }
    if let Some(query) = query.strip_prefix("crates ") {
        return Ok(handle_crates(query)?);
    }
    if let Some(query) = query.strip_prefix("ap ") {
        return Ok(handle_auspost(query)?);
    }
    Ok(Url::parse_with_params(
        "https://duckduckgo.com/?k1=-1",
        &[("q", query)],
    )?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("https://github.com/fullylegit", "gh @fullylegit")]
    #[test_case("https://github.com/fullylegit/ja3", "gh fullylegit/ja3")]
    #[test_case("https://github.com/search?q=test", "gh test")]
    #[test_case("https://github.com/rust-lang/rust/issues/1", "gh rust-lang/rust #1")]
    #[test_case("https://github.com/rust-lang/rust/pull/168", "gh rust-lang/rust !168")]
    #[test_case(
        "https://hub.docker.com/r/nvidia/k8s-device-plugin",
        "dh nvidia/k8s-device-plugin"
    )]
    #[test_case(
        "https://hub.docker.com/r/nvidia/k8s-device-plugin",
        "dh r/nvidia/k8s-device-plugin"
    )]
    #[test_case("https://hub.docker.com/search?q=test", "dh test")]
    #[test_case("https://weather.bom.gov.au/location/r3dp390-canberra", "weather")]
    #[test_case("https://en.wikipedia.org/wiki/Special:Search?search=test", "w test")]
    #[test_case(
        "https://en.wikipedia.org/wiki/Special:Search?search=test%2Flol",
        "w test/lol"
    )]
    #[test_case("https://stackoverflow.com/search?q=lol+donkey", "so lol donkey")]
    #[test_case("https://stackoverflow.com/search?q=lol%2Fdonkey", "so lol/donkey")]
    #[test_case("https://crates.io/search?q=lol%2Fdonkey", "crates lol/donkey")]
    #[test_case("https://auspost.com.au/mypost/track/#/details/ABC123", "ap ABC123")]
    #[test_case("https://hub.docker.com/_/nginx", "dh /nginx" ; "docker hub 1")]
    #[test_case("https://hub.docker.com/_/nginx", "dh _/nginx" ; "docker hub 2")]
    #[test_case("https://hub.docker.com/_/nginx", "dh r/nginx" ; "docker hub 3")]
    #[test_case("https://docs.rs/actix-web/", "docs actix-web" ; "docs.rs 1")]
    #[test_case("https://docs.rs/actix-web/", "docs/actix-web" ; "docs.rs 2")]
    #[test_case("https://docs.rs/actix-web/3.3.0", "docs actix-web/3.3.0" ; "docs.rs 3")]
    #[test_case("https://docs.rs/actix-web/3.3.0", "docs/actix-web/3.3.0" ; "docs.rs 4")]
    #[test_case("https://docs.rs/actix-web/3.3.0", "docs actix-web@3.3.0" ; "docs.rs 5")]
    #[test_case("https://docs.rs/actix-web/3.3.0", "docs/actix-web@3.3.0" ; "docs.rs 6")]
    // this is already handled by docs.rs but doing it here removes
    // an additional redirect
    #[test_case("https://doc.rust-lang.org/stable/std/", "docs std" ; "docs.rs 7")]
    #[test_case("https://doc.rust-lang.org/stable/std/", "docs/std" ; "docs.rs 8")]
    #[test_case("https://duckduckgo.com/?k1=-1&q=search", "search")]
    #[test_case("https://duckduckgo.com/?k1=-1&q=lol+donkey", "lol donkey")]
    #[test_case("https://duckduckgo.com/?k1=-1&q=lol%2Fdonkey", "lol/donkey")]
    #[test_case("https://this-week-in-rust.org/", "twir")]
    #[test_case("https://www.abc.net.au/news", "abc")]
    #[test_case("https://hackaday.com/blog/", "had")]
    #[test_case("https://slashdot.org/", "sd")]
    #[test_case("https://slashdot.org/", "/.")]
    #[test_case("https://www.servethehome.com/", "sth")]
    #[test_case("https://xkcd.com/", "x")]
    #[test_case("https://github.com/", "gh")]
    #[test_case("https://bushfire.io/", "bfio")]
    #[test_case("https://stackoverflow.com/search?q=search", "so search")]
    #[test_case("https://crates.io/search?q=search", "crates search")]
    #[test_case("https://crates.io/search?q=lol+donkey", "crates lol donkey")]
    fn run_tests(expected: &str, query: &str) -> Result<()> {
        let actual = query_to_url(query)?;
        assert_eq!(expected, actual.as_str(), "query: {:?}", query);
        Ok(())
    }
}
